use crate::callable::{core::*, keywords::*, operators::*};
/// Grammar Parsers
///
/// The primary interface for this module is `parse`. Internally, it dispatches
/// out to individual parsers for specific grammar tokens. Most grammar tokens
/// expect either a single `pest::iterators::Pair` or `pest::iterators::Pairs`,
/// and return a `RExpr`, with a few more specific internal parsers returning
/// `RExprList`s or tuples of parsed expressions.
///
use crate::error::Error;
use crate::internal_err;
use crate::lang::Signal;
use crate::object::{Expr, ExprList};

use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest::Parser;

pub type ParseResult = Result<Expr, Signal>;
pub type ParseListResult = Result<ExprList, Signal>;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            .op(Op::infix(assign, Right))
            .op(Op::infix(or, Left) | Op::infix(vor, Left))
            .op(Op::infix(and, Left) | Op::infix(vand, Left))
            .op(Op::infix(lt, Left) | Op::infix(gt, Left) | Op::infix(lte, Left) | Op::infix(gte, Left) | Op::infix(eq, Left) | Op::infix(neq, Left))
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
            .op(Op::infix(modulo, Left) | Op::infix(special, Left) | Op::infix(pipe, Left))
            .op(Op::infix(power, Left))
            .op(Op::infix(colon, Left))
            .op(Op::infix(dollar, Left))
   };
}

pub fn parse(s: &str) -> ParseResult {
    match RParser::parse(Rule::repl, s) {
        // comments currently entirely unparsed, but return NULL
        Ok(pairs) if pairs.len() == 0 => Err(Signal::Thunk),

        // for any expressions
        Ok(pairs) => parse_expr(pairs),
        Err(e) => Err(Error::ParseFailureVerbose(Box::new(e)).into()),
    }
}

pub fn parse_args(s: &str) -> ParseListResult {
    match RParser::parse(Rule::repl, s) {
        Ok(mut pairs) => parse_pairlist(pairs.next().map_or(internal_err!(), Ok)?),
        Err(e) => Err(Error::ParseFailureVerbose(Box::new(e)).into()),
    }
}

fn parse_expr(pairs: Pairs<Rule>) -> ParseResult {
    PRATT_PARSER
        .map_primary(parse_primary)
        .map_infix(|lhs, op, rhs| {
            // infix operator with two unnamed arguments
            let args = vec![(None, lhs?), (None, rhs?)].into();

            let op: Box<dyn Builtin> = match op.as_rule() {
                Rule::add => Box::new(InfixAdd),
                Rule::subtract => Box::new(InfixSub),
                Rule::multiply => Box::new(InfixMul),
                Rule::divide => Box::new(InfixDiv),
                Rule::dollar => Box::new(InfixDollar),
                Rule::power => Box::new(InfixPow),
                Rule::colon => Box::new(InfixColon),
                Rule::modulo => Box::new(InfixMod),
                Rule::assign => Box::new(InfixAssign),
                Rule::or => Box::new(InfixOr),
                Rule::and => Box::new(InfixAnd),
                Rule::vor => Box::new(InfixVectorOr),
                Rule::vand => Box::new(InfixVectorAnd),
                Rule::gt => Box::new(InfixGreater),
                Rule::lt => Box::new(InfixLess),
                Rule::gte => Box::new(InfixGreaterEqual),
                Rule::lte => Box::new(InfixLessEqual),
                Rule::eq => Box::new(InfixEqual),
                Rule::neq => Box::new(InfixNotEqual),
                Rule::pipe => Box::new(InfixPipe),
                rule => return Err(Error::ParseUnexpected(rule).into()),
            };

            Ok(Expr::Call(Box::new(Expr::Primitive(op)), args))
        })
        .parse(pairs)
}

fn parse_primary(pair: Pair<Rule>) -> ParseResult {
    match pair.as_rule() {
        // prefix and postfix notation
        Rule::postfixed => parse_postfixed(pair),
        Rule::prefixed => parse_prefixed(pair),

        // bracketed expression block
        Rule::expr => parse_expr(pair.into_inner()),
        Rule::block_exprs => parse_block(pair),

        // keyworded composite expressions
        Rule::kw_function => parse_function(pair),
        Rule::kw_while => parse_while(pair),
        Rule::kw_for => parse_for(pair),
        Rule::kw_if_else => parse_if_else(pair),
        Rule::kw_repeat => parse_repeat(pair),
        Rule::kw_break => Ok(Expr::Break),
        Rule::kw_continue => Ok(Expr::Continue),
        Rule::kw_return => parse_return(pair),

        // reserved values
        Rule::val_true => Ok(Expr::Bool(true)),
        Rule::val_false => Ok(Expr::Bool(false)),
        Rule::val_null => Ok(Expr::Null),
        Rule::val_na => Ok(Expr::NA),
        Rule::val_inf => Ok(Expr::Inf),

        // reserved symbols
        Rule::ellipsis => Ok(Expr::Ellipsis(None)),

        #[cfg(feature = "rest-args")]
        Rule::more => Ok(Expr::More),

        #[cfg(not(feature = "rest-args"))]
        Rule::more => Err(Error::FeatureDisabledRestArgs.into()),

        // atomic values
        Rule::number => Ok(Expr::Number(
            pair.as_str().parse::<f64>().map_or(internal_err!(), Ok)?,
        )),
        Rule::integer => Ok(Expr::Integer(
            pair.as_str().parse::<i32>().map_or(internal_err!(), Ok)?,
        )),
        Rule::single_quoted_string => Ok(Expr::String(String::from(pair.as_str()))),
        Rule::double_quoted_string => Ok(Expr::String(String::from(pair.as_str()))),

        // structured values
        Rule::vec => parse_vec(pair),
        Rule::list => parse_list(pair),

        // calls and symbols
        Rule::call => parse_call(pair),
        Rule::symbol_ident => parse_symbol(pair),
        Rule::symbol_backticked => Ok(Expr::Symbol(String::from(pair.as_str()))),

        // otherwise fail
        rule => Err(Error::ParseUnexpected(rule).into()),
    }
}

fn parse_block(pair: Pair<Rule>) -> ParseResult {
    // extract each inline expression, and treat as unnamed list
    let exprs: ExprList = pair
        .into_inner()
        .map(|i| parse_expr(i.into_inner()))
        .collect::<Result<_, _>>()?;

    // build call from symbol and list
    Ok(Expr::new_primitive_call(KeywordBlock, exprs))
}

fn parse_named(pair: Pair<Rule>) -> Result<(Option<String>, Expr), Signal> {
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    Ok((Some(name), parse_expr(inner)?))
}

pub fn parse_pairlist(pair: Pair<Rule>) -> ParseListResult {
    let exprs: ExprList = pair
        .into_inner()
        .map(|i| match i.as_rule() {
            Rule::named => parse_named(i),
            Rule::ellipsis => Ok((None, Expr::Ellipsis(None))),
            _ => Ok((None, parse_primary(i)?)),
        })
        .collect::<Result<_, _>>()?;

    Ok(exprs)
}

fn parse_call(pair: Pair<Rule>) -> ParseResult {
    let mut inner = pair.into_inner();
    let name = inner.next().map_or(internal_err!(), |i| Ok(i.as_str()))?;
    let pairs = parse_pairlist(inner.next().map_or(internal_err!(), Ok)?)?;

    match name {
        "list" => Ok(Expr::List(pairs)),
        name => Ok(Expr::Call(Box::new(Expr::String(name.to_string())), pairs)),
    }
}

fn parse_function(pair: Pair<Rule>) -> ParseResult {
    let mut inner = pair.into_inner();
    let params = parse_pairlist(inner.next().map_or(internal_err!(), Ok)?)?.as_formals();
    let body = parse_expr(inner)?;
    Ok(Expr::Function(params, Box::new(body)))
}

fn parse_if_else(pair: Pair<Rule>) -> ParseResult {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().map_or(internal_err!(), Ok)?.into_inner())?;
    let true_expr = parse_expr(inner.next().map_or(internal_err!(), Ok)?.into_inner())?;

    let false_expr = if let Some(false_block) = inner.next() {
        parse_expr(false_block.into_inner())?
    } else {
        Expr::Null
    };

    let args = ExprList::from(vec![cond, true_expr, false_expr]);
    Ok(Expr::new_primitive_call(KeywordIf, args))
}

fn parse_return(pair: Pair<Rule>) -> ParseResult {
    let mut inner = pair.into_inner();
    let expr = parse_expr(inner.next().map_or(internal_err!(), Ok)?.into_inner())?;
    let args = ExprList::from(vec![expr]);
    Ok(Expr::new_primitive_call(KeywordReturn, args))
}

fn parse_symbol(pair: Pair<Rule>) -> ParseResult {
    Ok(Expr::Symbol(String::from(pair.as_str())))
}

fn parse_for(pair: Pair<Rule>) -> ParseResult {
    let mut inner = pair.into_inner();

    let Expr::Symbol(var) = parse_symbol(inner.next().map_or(internal_err!(), Ok)?)? else {
        return internal_err!();
    };

    let iter = parse_expr(inner.next().map_or(internal_err!(), Ok)?.into_inner())?;
    let body = parse_expr(inner.next().map_or(internal_err!(), Ok)?.into_inner())?;

    let args = ExprList::from(vec![(Some(var), iter), (None, body)]);
    Ok(Expr::new_primitive_call(KeywordFor, args))
}

fn parse_while(pair: Pair<Rule>) -> ParseResult {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().map_or(internal_err!(), Ok)?.into_inner())?;
    let body = parse_expr(inner.next().map_or(internal_err!(), Ok)?.into_inner())?;
    let args = ExprList::from(vec![cond, body]);
    Ok(Expr::new_primitive_call(KeywordWhile, args))
}

fn parse_repeat(pair: Pair<Rule>) -> ParseResult {
    let mut inner = pair.into_inner();
    let body = parse_expr(inner.next().map_or(internal_err!(), Ok)?.into_inner())?;
    let args = ExprList::from(vec![body]);
    Ok(Expr::new_primitive_call(KeywordRepeat, args))
}

fn parse_postfix(pair: Pair<Rule>) -> Result<(Expr, ExprList), Signal> {
    match pair.as_rule() {
        Rule::call => Ok((Expr::Null, parse_pairlist(pair)?)),
        Rule::index => {
            let args = parse_pairlist(pair)?;
            Ok((Expr::as_primitive(PostfixIndex), args))
        }
        Rule::vector_index => Ok((Expr::as_primitive(PostfixVecIndex), parse_pairlist(pair)?)),

        #[cfg(feature = "rest-args")]
        Rule::more => {
            let val = pair.as_str();
            Ok((Expr::Ellipsis(Some(val.to_string())), ExprList::new()))
        }

        #[cfg(not(feature = "rest-args"))]
        Rule::more => Err(Error::FeatureDisabledRestArgs.into()),

        rule => Err(Error::ParseUnexpected(rule).into()),
    }
}

fn parse_postfixed(pair: Pair<Rule>) -> ParseResult {
    let mut inner = pair.into_inner();
    let mut result = parse_primary(inner.next().map_or(internal_err!(), Ok)?)?;

    for next in inner {
        let (what, mut args) = parse_postfix(next)?;
        result = match what {
            // Null used here has a magic value to dispatch on `x(...)` calls
            // if postfix is parenthesized pairlist, it's a call to result
            Expr::Null => Expr::Call(Box::new(result), args),

            // otherwise call to a postfix operator with result as the first arg
            _ => {
                args.insert(0, result);

                Expr::Call(Box::new(what), args)
            }
        };
    }

    Ok(result)
}

fn parse_prefixed(pair: Pair<Rule>) -> ParseResult {
    let mut inner = pair.into_inner().rev();
    let mut result = parse_postfixed(inner.next().map_or(internal_err!(), Ok)?)?;

    // iterate backwards through prefixes, applying prefixes from inside-out
    for prev in inner {
        result = match prev.as_rule() {
            Rule::subtract => {
                let args = ExprList::from(vec![result]);
                Expr::new_primitive_call(PrefixSub, args)
            }

            #[cfg(feature = "rest-args")]
            Rule::more => Expr::Ellipsis(Some(result.to_string())),

            #[cfg(not(feature = "rest-args"))]
            Rule::more => return Err(Error::FeatureDisabledRestArgs.into()),

            rule => return Err(Error::ParseUnexpected(rule).into()),
        }
    }

    Ok(result)
}

fn parse_vec(pair: Pair<Rule>) -> ParseResult {
    let args = parse_pairlist(pair)?;
    Ok(Expr::new_primitive_call(PrimVec, args))
}

fn parse_list(pair: Pair<Rule>) -> ParseResult {
    let args = parse_pairlist(pair)?;
    Ok(Expr::List(args))
}

#[cfg(test)]
mod test {
    use crate::r;

    #[test]
    fn prefix_with_space() {
        assert_eq! {
            r! {{"- 1"}},
            r! { -1 }
        }
    }

    #[test]
    fn postfix_with_space() {
        assert_eq! {
            r! {{"c (1)"}},
            r! {{"c(1)"}}
        }
    }
}
