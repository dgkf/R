use crate::callable::core::Builtin;
use crate::callable::{keywords::*, operators::*, primitive::PrimitiveList};
use r_derive::PrattOps;

/// Grammar Parsers
//
/// The primary interface for this module is `parse`. Internally, it dispatches
/// out to individual parsers for specific grammar tokens. Most grammar tokens
/// expect either a single `pest::iterators::Pair` or `pest::iterators::Pairs`,
/// and return a `RExpr`, with a few more specific internal parsers returning
/// `RExprList`s or tuples of parsed expressions.
///
use crate::error::RError;
use crate::lang::RSignal;
use crate::object::{Expr, ExprList};

use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest::{Parser, RuleType};

#[derive(Parser, Clone, Copy, PrattOps)]
#[grammar = "grammar/localizations/en.pest"]
#[grammar = "grammar/grammar.pest"]
pub struct ExprParser;

pub mod es {
    use r_derive::{PrattOps, Translate};
    #[derive(Parser, Clone, Copy, Translate, PrattOps)]
    #[grammar = "grammar/localizations/es.pest"]
    #[grammar = "grammar/grammar.pest"]
    pub struct ExprParser;
}

pub mod en {
    use r_derive::{PrattOps, Translate};
    #[derive(Parser, Clone, Copy, Translate, PrattOps)]
    #[grammar = "grammar/localizations/en.pest"]
    #[grammar = "grammar/grammar.pest"]
    pub struct ExprParser;
}

impl ExprParser {
    pub fn parse_input(s: &str) -> Result<Expr, RSignal> {
        match Self::parse(Rule::repl, s) {
            // comments currently entirely unparsed, return thunk
            Ok(pairs) if pairs.len() == 0 => Err(RSignal::Thunk),

            // for any expressions
            Ok(pairs) => Ok(parse_expr(&ExprParser, pratt_parser(), pairs)),
            Err(e) => Err(RError::ParseFailureVerbose(e).into()),
        }
    }
}

pub fn parse_with<P, R>(
    parser: &P,
    pratt: &PrattParser<R>,
    rule: R,
    input: &str,
) -> Result<Expr, RSignal>
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let pairs = P::parse(rule, input);
    match pairs {
        // comments currently entirely unparsed, return thunk
        Ok(pairs) if pairs.len() == 0 => Err(RSignal::Thunk),

        // for any expressions
        Ok(pairs) => Ok(parse_expr(parser, pratt, pairs)),
        Err(_e) => unreachable!(), // Err(RError::ParseFailureVerbose(e).into()),
    }
}

pub fn parse_expr<P, R>(parser: &P, pratt: &PrattParser<R>, pairs: Pairs<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    pratt
        .map_primary(|pair| parse_primary(parser, pratt, pair))
        .map_infix(|lhs, op, rhs| {
            // infix operator with two unnamed arguments
            let args = vec![(None, lhs), (None, rhs)].into();
            let op: Box<dyn Builtin> = match op.as_rule().into() {
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
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };

            Expr::Call(Box::new(Expr::Primitive(op)), args)
        })
        .parse(pairs)
}

fn parse_primary<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    match pair.as_rule().into() {
        // prefix and postfix notation
        Rule::postfixed => parse_postfixed(parser, pratt, pair),
        Rule::prefixed => parse_prefixed(parser, pratt, pair),

        // bracketed expression block
        Rule::expr => parse_expr(parser, pratt, pair.into_inner()),
        Rule::block_exprs => parse_block(parser, pratt, pair),

        // keyworded composite expressions
        Rule::kw_function => parse_function(parser, pratt, pair),
        Rule::kw_while => parse_while(parser, pratt, pair),
        Rule::kw_for => parse_for(parser, pratt, pair),
        Rule::kw_if_else => parse_if_else(parser, pratt, pair),
        Rule::kw_repeat => parse_repeat(parser, pratt, pair),
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
        Rule::vec => parse_vec(parser, pratt, pair),
        Rule::list => parse_list(parser, pratt, pair),

        // calls and symbols
        Rule::call => parse_call(parser, pratt, pair),
        Rule::symbol_ident => parse_symbol(parser, pratt, pair),
        Rule::symbol_backticked => Expr::Symbol(String::from(pair.as_str())),

        // otherwise fail
        rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
    }
}

fn parse_block<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    // extract each inline expression, and treat as unnamed list
    let exprs = pair
        .into_inner()
        .map(|i| parse_expr(parser, pratt, i.into_inner()))
        .collect();

    // build call from symbol and list
    Expr::new_primitive_call(KeywordBlock, exprs)
}

fn parse_named<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> (Option<String>, Expr)
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    (Some(name), parse_expr(parser, pratt, inner))
}

fn parse_pairlist<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> ExprList
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let exprs = pair
        .into_inner()
        .map(|i| match i.as_rule().into() {
            Rule::named => parse_named(parser, pratt, i),
            Rule::ellipsis => (None, Expr::Ellipsis),
            _ => (None, parse_primary(parser, pratt, i)),
        })
        .collect();

    exprs
}

fn parse_call<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    Expr::Call(
        Box::new(Expr::String(name)),
        parse_pairlist(parser, pratt, inner.next().unwrap()),
    )
}

fn parse_function<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let mut inner = pair.into_inner();
    let params = parse_pairlist(parser, pratt, inner.next().unwrap()).as_formals();
    let body = parse_expr(parser, pratt, inner);
    Expr::Function(params, Box::new(body))
}

fn parse_if_else<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let mut inner = pair.into_inner();
    let cond = parse_expr(parser, pratt, inner.next().unwrap().into_inner());
    let true_expr = parse_expr(parser, pratt, inner.next().unwrap().into_inner());

    let false_expr = if let Some(false_block) = inner.next() {
        parse_expr(parser, pratt, false_block.into_inner())
    } else {
        Expr::Null
    };

    let args = ExprList::from(vec![cond, true_expr, false_expr]);
    Expr::new_primitive_call(KeywordIf, args)
}

fn parse_symbol<P, R>(_parser: &P, _pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    Expr::Symbol(String::from(pair.as_str()))
}

fn parse_for<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let mut inner = pair.into_inner();

    let Expr::Symbol(var) = parse_symbol(parser, pratt, inner.next().unwrap()) else {
        unreachable!();
    };

    let iter = parse_expr(parser, pratt, inner.next().unwrap().into_inner());
    let body = parse_expr(parser, pratt, inner.next().unwrap().into_inner());

    let args = ExprList::from(vec![(Some(var), iter), (None, body)]);
    Expr::new_primitive_call(KeywordFor, args)
}

fn parse_while<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let mut inner = pair.into_inner();
    let cond = parse_expr(parser, pratt, inner.next().unwrap().into_inner());
    let body = parse_expr(parser, pratt, inner.next().unwrap().into_inner());
    let args = ExprList::from(vec![cond, body]);
    Expr::new_primitive_call(KeywordWhile, args)
}

fn parse_repeat<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let mut inner = pair.into_inner();
    let body = parse_expr(parser, pratt, inner.next().unwrap().into_inner());
    let args = ExprList::from(vec![body]);
    Expr::new_primitive_call(KeywordRepeat, args)
}

fn parse_postfix<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> (Expr, ExprList)
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    match pair.as_rule().into() {
        Rule::call => (Expr::Null, parse_pairlist(parser, pratt, pair)),
        Rule::index => {
            let args = parse_pairlist(parser, pratt, pair);
            (Expr::as_primitive(PostfixIndex), args)
        }
        Rule::vector_index => (
            Expr::as_primitive(PostfixVecIndex),
            parse_pairlist(parser, pratt, pair),
        ),
        rule => unreachable!("invalid postfix operator '{:#?}'", rule),
    }
}

fn parse_postfixed<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let mut inner = pair.into_inner();
    let mut result = parse_primary(parser, pratt, inner.next().unwrap());

    while let Some(next) = inner.next() {
        let (what, mut args) = parse_postfix(parser, pratt, next);
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

fn parse_prefixed<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let mut inner = pair.into_inner().rev();
    let mut result = parse_postfixed(parser, pratt, inner.next().unwrap());

    // iterate backwards through prefixes, applying prefixes from inside-out
    while let Some(prev) = inner.next() {
        result = match prev.as_rule().into() {
            Rule::subtract => {
                let args = ExprList::from(vec![result]);
                Expr::new_primitive_call(PrefixSub, args)
            }
            _ => unreachable!("invalid prefix operator '{:#?}'", prev),
        }
    }

    result
}

fn parse_vec<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let args = parse_pairlist(parser, pratt, pair);
    Expr::new_primitive_call(PrimVec, args)
}

fn parse_list<P, R>(parser: &P, pratt: &PrattParser<R>, pair: Pair<R>) -> Expr
where
    P: Parser<R>,
    R: RuleType + Into<Rule>,
{
    let args = parse_pairlist(parser, pratt, pair);
    Expr::new_primitive_call(PrimitiveList, args)
}
