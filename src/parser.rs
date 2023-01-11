use crate::ast::*;
use crate::builtins::*;
use crate::error::RError;
use crate::r_repl::highlight::RHighlights;

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
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
   };
}

pub fn parse_block(pair: Pair<Rule>) -> RExpr {
    // extract each inline expression, and treat as unnamed list
    let exprs = pair
        .into_inner()
        .map(|i| parse_expr(i.into_inner()))
        .collect();

    // build call from symbol and list
    RExpr::Call(Box::new(RExprBlock), exprs)
}

pub fn parse_named(pair: Pair<Rule>) -> (Option<String>, RExpr) {
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    (Some(name), parse_expr(inner))
}

pub fn parse_list(pair: Pair<Rule>) -> RExprList {
    let exprs = pair
        .into_inner()
        .map(|i| match i.as_rule() {
            Rule::named => parse_named(i),
            Rule::expr | Rule::inline | Rule::block => (None, parse_expr(i.into_inner())),
            Rule::ellipsis => (None, RExpr::Ellipsis),
            rule => unreachable!("Expected named or unnamed arguments, found {:?}", rule),
        })
        .collect();

    exprs
}

pub fn parse_call(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    RExpr::Call(Box::new(name), parse_list(inner.next().unwrap()))
}

pub fn parse_function(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let params = parse_list(inner.next().unwrap());
    let body = parse_expr(inner);
    RExpr::Function(params, Box::new(body))
}

pub fn parse_if_else(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().unwrap().into_inner());
    let true_expr = parse_expr(inner.next().unwrap().into_inner());

    let false_expr = if let Some(false_block) = inner.next() {
        parse_expr(false_block.into_inner())
    } else {
        RExpr::Null
    };

    RExpr::Call(
        Box::new(RExprIf),
        RExprList::from(vec![(None, cond), (None, true_expr), (None, false_expr)]),
    )
}

pub fn parse_symbol(pair: Pair<Rule>) -> RExpr {
    RExpr::Symbol(String::from(pair.as_str()))
}

pub fn parse_for(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();

    let RExpr::Symbol(var) = parse_symbol(inner.next().unwrap()) else {
        unreachable!();
    };

    let iter = parse_expr(inner.next().unwrap().into_inner());
    let body = parse_expr(inner.next().unwrap().into_inner());

    RExpr::Call(
        Box::new(RExprFor),
        RExprList::from(vec![(Some(var), iter), (None, body)]),
    )
}

fn parse_while(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().unwrap().into_inner());
    let body = parse_expr(inner.next().unwrap().into_inner());
    RExpr::Call(Box::new(RExprWhile), RExprList::from(vec![cond, body]))
}

fn parse_repeat(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let body = parse_expr(inner.next().unwrap().into_inner());
    RExpr::Call(Box::new(RExprRepeat), RExprList::from(vec![body]))
}

pub fn parse_expr(pairs: Pairs<Rule>) -> RExpr {
    PRATT_PARSER
        .map_primary(|primary| {
            match primary.as_rule() {
                // language parsing rules
                Rule::kw_break => RExpr::Break,
                Rule::kw_continue => RExpr::Continue,
                Rule::null => RExpr::Null,
                Rule::na => RExpr::NA,
                Rule::inf => RExpr::Inf,
                Rule::ellipsis => RExpr::Ellipsis,
                Rule::kw_function => parse_function(primary),
                Rule::kw_while => parse_while(primary),
                Rule::kw_for => parse_for(primary),
                Rule::kw_if_else => parse_if_else(primary),
                Rule::kw_repeat => parse_repeat(primary),
                Rule::call => parse_call(primary),
                Rule::expr => parse_expr(primary.into_inner()),
                Rule::inline => parse_expr(primary.into_inner()),
                Rule::block_inline => parse_expr(primary.into_inner()),
                Rule::block => parse_block(primary),
                Rule::list => RExpr::List(parse_list(primary)),
                Rule::boolean_true => RExpr::Bool(true),
                Rule::boolean_false => RExpr::Bool(false),
                Rule::number => RExpr::Number(primary.as_str().parse::<f64>().unwrap()),
                Rule::integer => RExpr::Integer(primary.as_str().parse::<i32>().unwrap()),
                Rule::string_expr => parse_expr(primary.into_inner()), // TODO: improve grammar to avoid unnecessary parse
                Rule::string => RExpr::String(String::from(primary.as_str())),
                Rule::symbol_ident => parse_symbol(primary),
                Rule::symbol_backticked => RExpr::Symbol(String::from(primary.as_str())),

                // otherwise fail
                rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
            }
        })
        .map_infix(|lhs, op, rhs| {
            // infix operator with two unnamed arguments
            let args = vec![(None, lhs), (None, rhs)].into();

            let op: Box<dyn Callable> = match op.as_rule() {
                Rule::add => Box::new(InfixAdd),
                Rule::subtract => Box::new(InfixSub),
                Rule::multiply => Box::new(InfixMul),
                Rule::divide => Box::new(InfixDiv),
                Rule::assign => Box::new(InfixAssign),
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };

            RExpr::Call(op, args)
        })
        .parse(pairs)
}

pub fn parse(s: &str) -> Result<RExpr, RError> {
    match RParser::parse(Rule::repl, s) {
        Ok(mut pairs) => {
            let inner = pairs.next().unwrap().into_inner();
            Ok(parse_expr(inner))
        }
        Err(e) => Err(RError::ParseFailureVerbose(e)),
    }
}

pub fn parse_hl(pairs: Pairs<Rule>) -> RHighlights {
    println!("{:#?}", pairs);
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::hl_num => RHighlights::Number,
            _ => RHighlights::None,
        })
        .parse(pairs)
}

pub fn parse_highlights(s: &str) -> Result<RHighlights, RError> {
    match RParser::parse(Rule::hl, s) {
        Ok(mut pairs) => {
            let inner = pairs.next().unwrap().into_inner();
            Ok(parse_hl(inner))
        }
        Err(e) => Err(RError::ParseFailureVerbose(e)),
    }
}
