/// Grammar Parsers
///
/// The primary interface for this module is `parse`. Internally, it dispatches
/// out to individual parsers for specific grammar tokens. Most grammar tokens
/// expect either a single `pest::iterators::Pair` or `pest::iterators::Pairs`,
/// and return a `RExpr`, with a few more specific internal parsers returning
/// `RExprList`s or tuples of parsed expressions.
///
use crate::ast::*;
use crate::error::RError;
use crate::callable::{core::*, keywords::*, operators::*, primitive::PrimitiveList};

use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest::Parser;

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
   };
}

pub fn parse(s: &str) -> Result<Expr, RError> {
    match RParser::parse(Rule::repl, s) {
        Ok(pairs) => Ok(parse_expr(pairs)),
        Err(e) => Err(RError::ParseFailureVerbose(e)),
    }
}

pub fn parse_args(s: &str) -> Result<ExprList, RError> {
    match RParser::parse(Rule::repl, s) {
        Ok(mut pairs) => Ok(parse_pairlist(pairs.next().unwrap())),
        Err(e) => Err(RError::ParseFailureVerbose(e)),
    }
}

fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(parse_primary)
        .map_infix(|lhs, op, rhs| {
            // infix operator with two unnamed arguments
            let args = vec![(None, lhs), (None, rhs)].into();

            let op: Box<dyn Primitive> = match op.as_rule() {
                Rule::add => Box::new(InfixAdd),
                Rule::subtract => Box::new(InfixSub),
                Rule::multiply => Box::new(InfixMul),
                Rule::divide => Box::new(InfixDiv),
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
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };

            Expr::Call(Box::new(Expr::Primitive(op)), args)
        })
        .parse(pairs)
}

fn parse_primary(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        // prefix and postfix notation
        Rule::postfixed => parse_postfixed(pair),
        Rule::prefixed => parse_prefixed(pair),

        // bracketed expression block
        Rule::block => parse_block(pair),
        Rule::exprs => parse_block(pair),
        Rule::expr => parse_expr(pair.into_inner()),

        // keyworded composite expressions
        Rule::kw_function => parse_function(pair),
        Rule::kw_while => parse_while(pair),
        Rule::kw_for => parse_for(pair),
        Rule::kw_if_else => parse_if_else(pair),
        Rule::kw_repeat => parse_repeat(pair),
        Rule::kw_break => Expr::Break,
        Rule::kw_continue => Expr::Continue,

        // reserved values
        Rule::val_true => Expr::Bool(true),
        Rule::val_false => Expr::Bool(false),
        Rule::val_null => Expr::Null,
        Rule::val_na => Expr::NA,
        Rule::val_inf => Expr::Inf,

        // reserved symbols
        Rule::ellipsis => Expr::Ellipsis,

        // atomic values
        Rule::number => Expr::Number(pair.as_str().parse::<f64>().unwrap()),
        Rule::integer => Expr::Integer(pair.as_str().parse::<i32>().unwrap()),
        Rule::single_quoted_string => Expr::String(String::from(pair.as_str())),
        Rule::double_quoted_string => Expr::String(String::from(pair.as_str())),

        // structured values
        Rule::vec => parse_vec(pair),
        Rule::list => parse_list(pair),

        // calls and symbols
        Rule::call => parse_call(pair),
        Rule::symbol_ident => parse_symbol(pair),
        Rule::symbol_backticked => Expr::Symbol(String::from(pair.as_str())),

        // otherwise fail
        rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
    }
}

fn parse_block(pair: Pair<Rule>) -> Expr {
    // extract each inline expression, and treat as unnamed list
    let exprs = pair
        .into_inner()
        .map(|i| parse_expr(i.into_inner()))
        .collect();

    // build call from symbol and list
    Expr::new_primitive_call(PrimBlock, exprs)
}

fn parse_named(pair: Pair<Rule>) -> (Option<String>, Expr) {
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    (Some(name), parse_expr(inner))
}

fn parse_pairlist(pair: Pair<Rule>) -> ExprList {
    let exprs = pair
        .into_inner()
        .map(|i| match i.as_rule() {
            Rule::named => parse_named(i),
            Rule::ellipsis => (None, Expr::Ellipsis),
            _ => (None, parse_primary(i)),
        })
        .collect();

    exprs
}

fn parse_call(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    Expr::Call(
        Box::new(Expr::String(name)),
        parse_pairlist(inner.next().unwrap()),
    )
}

fn parse_function(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let params = parse_pairlist(inner.next().unwrap()).as_formals();
    let body = parse_expr(inner);
    Expr::Function(params, Box::new(body))
}

fn parse_if_else(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().unwrap().into_inner());
    let true_expr = parse_expr(inner.next().unwrap().into_inner());

    let false_expr = if let Some(false_block) = inner.next() {
        parse_expr(false_block.into_inner())
    } else {
        Expr::Null
    };

    let args = ExprList::from(vec![cond, true_expr, false_expr]);
    Expr::new_primitive_call(PrimIf, args)
}

fn parse_symbol(pair: Pair<Rule>) -> Expr {
    Expr::Symbol(String::from(pair.as_str()))
}

fn parse_for(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let Expr::Symbol(var) = parse_symbol(inner.next().unwrap()) else {
        unreachable!();
    };

    let iter = parse_expr(inner.next().unwrap().into_inner());
    let body = parse_expr(inner.next().unwrap().into_inner());

    let args = ExprList::from(vec![(Some(var), iter), (None, body)]);
    Expr::new_primitive_call(PrimFor, args)
}

fn parse_while(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().unwrap().into_inner());
    let body = parse_expr(inner.next().unwrap().into_inner());
    let args = ExprList::from(vec![cond, body]);
    Expr::new_primitive_call(PrimWhile, args)
}

fn parse_repeat(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let body = parse_expr(inner.next().unwrap().into_inner());
    let args = ExprList::from(vec![body]);
    Expr::new_primitive_call(PrimRepeat, args)
}

fn parse_postfix(pair: Pair<Rule>) -> (Expr, ExprList) {
    match pair.as_rule() {
        Rule::call => (Expr::Null, parse_pairlist(pair)),
        Rule::index => {
            let args = parse_pairlist(pair);
            (Expr::as_primitive(PostfixIndex), args)
        }
        Rule::vector_index => (Expr::as_primitive(PostfixVecIndex), parse_pairlist(pair)),
        rule => unreachable!("invalid postfix operator '{:#?}'", rule),
    }
}

fn parse_postfixed(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut result = parse_primary(inner.next().unwrap());

    while let Some(next) = inner.next() {
        let (what, mut args) = parse_postfix(next);
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

    result
}

fn parse_prefixed(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner().rev();
    let mut result = parse_postfixed(inner.next().unwrap());

    // iterate backwards through prefixes, applying prefixes from inside-out
    while let Some(prev) = inner.next() {
        result = match prev.as_rule() {
            Rule::subtract => {
                let args = ExprList::from(vec![result]);
                Expr::new_primitive_call(PrefixSub, args)
            }
            _ => unreachable!("invalid prefix operator '{:#?}'", prev),
        }
    }

    result
}

fn parse_vec(pair: Pair<Rule>) -> Expr {
    let args = parse_pairlist(pair);
    Expr::new_primitive_call(PrimVec, args)
}

fn parse_list(pair: Pair<Rule>) -> Expr {
    let args = parse_pairlist(pair);
    Expr::new_primitive_call(PrimitiveList, args)
}
