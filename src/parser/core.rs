/// Grammar Parsers
///
/// The primary interface for this module is `parse`. Internally, it dispatches
/// out to individual parsers for specific grammar tokens. Most grammar tokens
/// expect either a single `pest::iterators::Pair` or `pest::iterators::Pairs`,
/// and return a `RExpr`, with a few more specific internal parsers returning
/// `RExprList`s or tuples of parsed expressions.
///
use crate::callable::{core::*, keywords::*, operators::*};
use crate::cli::Experiment;
use crate::error::Error;
use crate::internal_err;
use crate::lang::Signal;
use crate::object::{Expr, ExprList};
use crate::parser::*;
use crate::session::SessionParserConfig;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest::{Parser, RuleType};

pub type ParseResult = Result<Expr, Signal>;
pub type ParseListResult = Result<ExprList, Signal>;

pub fn parse_expr<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pairs: Pairs<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    pratt
        .map_primary(|pair| parse_primary(config, parser, pratt, pair))
        .map_infix(|lhs, op, rhs| {
            // infix operator with two unnamed arguments
            let args = vec![(None, lhs?), (None, rhs?)].into();
            let op: Box<dyn Builtin> = match op.as_rule().into() {
                en::Rule::add => Box::new(InfixAdd),
                en::Rule::subtract => Box::new(InfixSub),
                en::Rule::multiply => Box::new(InfixMul),
                en::Rule::divide => Box::new(InfixDiv),
                en::Rule::dollar => Box::new(InfixDollar),
                en::Rule::power => Box::new(InfixPow),
                en::Rule::colon => Box::new(InfixColon),
                en::Rule::modulo => Box::new(InfixMod),
                en::Rule::assign => Box::new(InfixAssign),
                en::Rule::or => Box::new(InfixOr),
                en::Rule::and => Box::new(InfixAnd),
                en::Rule::vor => Box::new(InfixVectorOr),
                en::Rule::vand => Box::new(InfixVectorAnd),
                en::Rule::gt => Box::new(InfixGreater),
                en::Rule::lt => Box::new(InfixLess),
                en::Rule::gte => Box::new(InfixGreaterEqual),
                en::Rule::lte => Box::new(InfixLessEqual),
                en::Rule::eq => Box::new(InfixEqual),
                en::Rule::neq => Box::new(InfixNotEqual),
                en::Rule::pipe => Box::new(InfixPipe),
                rule => {
                    let span = (op.as_span().start(), op.as_span().end());
                    return Err(Error::ParseUnexpected(rule, span).into());
                }
            };

            Ok(Expr::Call(Box::new(Expr::Primitive(op)), args))
        })
        .parse(pairs)
}

fn parse_primary<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    match pair.as_rule().into() {
        // prefix and postfix notation
        en::Rule::postfixed => parse_postfixed(config, parser, pratt, pair),
        en::Rule::prefixed => parse_prefixed(config, parser, pratt, pair),

        // bracketed expression block
        en::Rule::expr => parse_expr(config, parser, pratt, pair.into_inner()),
        en::Rule::paren_expr => parse_paren(config, parser, pratt, pair),
        en::Rule::block_exprs => parse_block(config, parser, pratt, pair),

        // keyworded composite expressions
        en::Rule::kw_function => parse_function(config, parser, pratt, pair),
        en::Rule::kw_while => parse_while(config, parser, pratt, pair),
        en::Rule::kw_for => parse_for(config, parser, pratt, pair),
        en::Rule::kw_if_else => parse_if_else(config, parser, pratt, pair),
        en::Rule::kw_repeat => parse_repeat(config, parser, pratt, pair),
        en::Rule::kw_break => Ok(Expr::Break),
        en::Rule::kw_continue => Ok(Expr::Continue),
        en::Rule::kw_return => parse_return(config, parser, pratt, pair),

        // reserved values
        en::Rule::val_true => Ok(Expr::Bool(true)),
        en::Rule::val_false => Ok(Expr::Bool(false)),
        en::Rule::val_null => Ok(Expr::Null),
        en::Rule::val_na => Ok(Expr::NA),
        en::Rule::val_inf => Ok(Expr::Inf),

        // reserved symbols
        en::Rule::more => Ok(Expr::More),

        // atomic values
        en::Rule::number => Ok(Expr::Number(
            pair.as_str()
                .replace('_', "")
                .parse::<f64>()
                .map_or(internal_err!(), Ok)?,
        )),
        en::Rule::integer => Ok(Expr::Integer(
            pair.as_str()
                .replace('_', "")
                .parse::<i32>()
                .map_or(internal_err!(), Ok)?,
        )),
        en::Rule::single_quoted_string => Ok(Expr::String(String::from(pair.as_str()))),
        en::Rule::double_quoted_string => Ok(Expr::String(String::from(pair.as_str()))),

        // structured values
        en::Rule::vec => parse_vec(config, parser, pratt, pair),
        en::Rule::list => parse_list(config, parser, pratt, pair),

        // calls and symbols
        en::Rule::call => parse_call(config, parser, pratt, pair),
        en::Rule::symbol_ident => parse_symbol(config, parser, pratt, pair),
        en::Rule::symbol_backticked => Ok(Expr::Symbol(String::from(pair.as_str()))),

        // otherwise fail
        rule => {
            let span = (pair.as_span().start(), pair.as_span().end());
            Err(Error::ParseUnexpected(rule, span).into())
        }
    }
}

fn parse_paren<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    // extract each inline expression, and treat as unnamed list
    let exprs: ExprList = pair
        .into_inner()
        .map(|i| parse_expr(config, parser, pratt, i.into_inner()))
        .collect::<Result<_, _>>()?;

    // build call from symbol and list
    Ok(Expr::new_primitive_call(KeywordParen, exprs))
}

fn parse_block<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    // extract each inline expression, and treat as unnamed list
    let exprs: ExprList = pair
        .into_inner()
        .map(|i| parse_expr(config, parser, pratt, i.into_inner()))
        .collect::<Result<_, _>>()?;

    // build call from symbol and list
    Ok(Expr::new_primitive_call(KeywordBlock, exprs))
}

fn parse_named<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> Result<(Option<String>, Expr), Signal>
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    Ok((Some(name), parse_expr(config, parser, pratt, inner)?))
}

fn parse_list_elements<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseListResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let exprs: ExprList = pair
        .into_inner()
        .map(|i| match i.as_rule().into() {
            en::Rule::named => parse_named(config, parser, pratt, i),
            _ => Ok((None, parse_primary(config, parser, pratt, i)?)),
        })
        .collect::<Result<_, _>>()?;

    Ok(exprs)
}

fn parse_call<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner();
    let name = inner.next().map_or(internal_err!(), |i| Ok(i.as_str()))?;
    let pairs = parse_list_elements(
        config,
        parser,
        pratt,
        inner.next().map_or(internal_err!(), Ok)?,
    )?;

    match name {
        "list" => Ok(Expr::List(pairs)),
        name => Ok(Expr::Call(Box::new(Expr::String(name.to_string())), pairs)),
    }
}

fn parse_function<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner();
    let params = parse_list_elements(
        config,
        parser,
        pratt,
        inner.next().map_or(internal_err!(), Ok)?,
    )?
    .as_formals();
    let body = parse_expr(config, parser, pratt, inner)?;
    Ok(Expr::Function(params, Box::new(body)))
}

fn parse_if_else<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner();

    let inner_cond = inner.next().map_or(internal_err!(), Ok)?.into_inner();
    let cond = parse_expr(config, parser, pratt, inner_cond)?;

    let inner_true = inner.next().map_or(internal_err!(), Ok)?.into_inner();
    let true_expr = parse_expr(config, parser, pratt, inner_true)?;

    let false_expr = if let Some(false_block) = inner.next() {
        parse_expr(config, parser, pratt, false_block.into_inner())?
    } else {
        Expr::Null
    };

    let args = ExprList::from(vec![cond, true_expr, false_expr]);
    Ok(Expr::new_primitive_call(KeywordIf, args))
}

fn parse_return<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner();
    let inner_expr = inner.next().map_or(internal_err!(), Ok)?.into_inner();
    let expr = parse_expr(config, parser, pratt, inner_expr)?;
    let args = ExprList::from(vec![expr]);
    Ok(Expr::new_primitive_call(KeywordReturn, args))
}

fn parse_symbol<P, R>(
    _config: &SessionParserConfig,
    _parser: &P,
    _pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    Ok(Expr::Symbol(String::from(pair.as_str())))
}

fn parse_for<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner();

    let inner_sym = inner.next().map_or(internal_err!(), Ok)?;
    let Expr::Symbol(var) = parse_symbol(config, parser, pratt, inner_sym)? else {
        return internal_err!();
    };

    let inner_iter = inner.next().map_or(internal_err!(), Ok)?.into_inner();
    let iter = parse_expr(config, parser, pratt, inner_iter)?;

    let inner_body = inner.next().map_or(internal_err!(), Ok)?.into_inner();
    let body = parse_expr(config, parser, pratt, inner_body)?;

    let args = ExprList::from(vec![(Some(var), iter), (None, body)]);
    Ok(Expr::new_primitive_call(KeywordFor, args))
}

fn parse_while<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner();
    let inner_cond = inner.next().map_or(internal_err!(), Ok)?.into_inner();
    let cond = parse_expr(config, parser, pratt, inner_cond)?;
    let inner_body = inner.next().map_or(internal_err!(), Ok)?.into_inner();
    let body = parse_expr(config, parser, pratt, inner_body)?;
    let args = ExprList::from(vec![cond, body]);
    Ok(Expr::new_primitive_call(KeywordWhile, args))
}

fn parse_repeat<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner();
    let inner_body = inner.next().map_or(internal_err!(), Ok)?.into_inner();
    let body = parse_expr(config, parser, pratt, inner_body)?;
    let args = ExprList::from(vec![body]);
    Ok(Expr::new_primitive_call(KeywordRepeat, args))
}

fn parse_postfix<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> Result<(Expr, ExprList), Signal>
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    match pair.as_rule().into() {
        en::Rule::call => Ok((
            Expr::Null,
            parse_list_elements(config, parser, pratt, pair)?,
        )),
        en::Rule::index => {
            let args = parse_list_elements(config, parser, pratt, pair)?;
            Ok((Expr::as_primitive(PostfixIndex), args))
        }
        en::Rule::vector_index => Ok((
            Expr::as_primitive(PostfixVecIndex),
            parse_list_elements(config, parser, pratt, pair)?,
        )),

        en::Rule::more => {
            let val = pair.as_str();
            let is_ellipsis = val == ".";
            if config.experiments.contains(&Experiment::RestArgs) {
                Ok((Expr::Ellipsis(Some(val.to_string())), ExprList::new()))
            } else if is_ellipsis {
                Ok((Expr::Ellipsis(None), ExprList::new()))
            } else {
                Err(Error::FeatureDisabledRestArgs.into())
            }
        }

        rule => {
            let span = (pair.as_span().start(), pair.as_span().end());
            Err(Error::ParseUnexpected(rule, span).into())
        }
    }
}

fn parse_postfixed<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner();
    let inner_next = inner.next().map_or(internal_err!(), Ok)?;
    let mut result = parse_primary(config, parser, pratt, inner_next)?;

    for next in inner {
        let (what, mut args) = parse_postfix(config, parser, pratt, next)?;
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

fn parse_prefixed<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let mut inner = pair.into_inner().rev();
    let inner_next = inner.next().map_or(internal_err!(), Ok)?;
    let mut result = parse_postfixed(config, parser, pratt, inner_next)?;

    // iterate backwards through prefixes, applying prefixes from inside-out
    for prev in inner {
        result = match prev.as_rule().into() {
            en::Rule::subtract => {
                let args = ExprList::from(vec![result]);
                Expr::new_primitive_call(PrefixSub, args)
            }

            en::Rule::not => {
                let args = ExprList::from(vec![result]);
                Expr::new_primitive_call(PrefixNot, args)
            }

            en::Rule::more => {
                let is_ellipsis = result.to_string() == ".";
                if config.experiments.contains(&Experiment::RestArgs) {
                    Expr::Ellipsis(Some(result.to_string()))
                } else if is_ellipsis {
                    Expr::Ellipsis(None)
                } else {
                    return Err(Error::FeatureDisabledRestArgs.into());
                }
            }

            rule => {
                let span = (prev.as_span().start(), prev.as_span().end());
                return Err(Error::ParseUnexpected(rule, span).into());
            }
        }
    }

    Ok(result)
}

fn parse_vec<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let args = parse_list_elements(config, parser, pratt, pair)?;
    Ok(Expr::new_primitive_call(KeywordVec, args))
}

fn parse_list<P, R>(
    config: &SessionParserConfig,
    parser: &P,
    pratt: &PrattParser<R>,
    pair: Pair<R>,
) -> ParseResult
where
    P: Parser<R> + LocalizedParser,
    R: RuleType + Into<en::Rule>,
{
    let args = parse_list_elements(config, parser, pratt, pair)?;
    Ok(Expr::new_primitive_call(KeywordList, args))
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

    #[test]
    fn separation_integer() {
        assert_eq! {
            r! {{"1_200_000L"}},
            r! {{"1200000L"}}
        }
    }

    #[test]
    fn separation_double_trailing() {
        assert_eq! {
            r! {{".000_123"}},
            r! {{".000123"}}
        }
    }
    #[test]
    fn separation_double_leading() {
        assert_eq! {
            r! {{"1_2.0_1"}},
            r! {{"1_2.01"}}
        }
    }
    #[test]
    fn separation_double_leading_zero() {
        assert_eq! {
            r! {{"0.000_123"}},
            r! {{"0.000123"}}
        }
    }
}
