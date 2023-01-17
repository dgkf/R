/// Grammar Parsers
///
/// The primary interface for this module is `parse`. Internally, it dispatches
/// out to individual parsers for specific grammar tokens. Most grammar tokens
/// expect either a single `pest::iterators::Pair` or `pest::iterators::Pairs`,
/// and return a `RExpr`, with a few more specific internal parsers returning
/// `RExprList`s or tuples of parsed expressions.
///
use crate::ast::*;
use crate::builtins::*;
use crate::error::RError;

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
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
            .op(Op::infix(modulo, Left) | Op::infix(special, Left) | Op::infix(pipe, Left))
            .op(Op::infix(power, Left))
   };
}

pub fn parse(s: &str) -> Result<RExpr, RError> {
    match RParser::parse(Rule::repl, s) {
        Ok(pairs) => Ok(parse_expr(pairs)),
        Err(e) => Err(RError::ParseFailureVerbose(e)),
    }
}

fn parse_expr(pairs: Pairs<Rule>) -> RExpr {
    PRATT_PARSER
        .map_primary(parse_primary)
        .map_infix(|lhs, op, rhs| {
            // infix operator with two unnamed arguments
            let args = vec![(None, lhs), (None, rhs)].into();

            let op: Box<dyn Callable> = match op.as_rule() {
                Rule::add => Box::new(InfixAdd),
                Rule::subtract => Box::new(InfixSub),
                Rule::multiply => Box::new(InfixMul),
                Rule::divide => Box::new(InfixDiv),
                Rule::power => Box::new(InfixPow),
                Rule::modulo => Box::new(InfixMod),
                Rule::assign => Box::new(InfixAssign),
                Rule::or => Box::new(InfixOr),
                Rule::and => Box::new(InfixAnd),
                Rule::vor => Box::new(InfixVectorOr),
                Rule::vand => Box::new(InfixVectorAnd),
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };

            RExpr::Call(Box::new(RExpr::Primitive(op)), args)
        })
        .parse(pairs)
}

fn parse_primary(pair: Pair<Rule>) -> RExpr {
    // println!("{:#?}", pair.as_rule());

    match pair.as_rule() {
        // prefix and postfix notation
        Rule::postfixed => parse_postfixed(pair),
        Rule::prefixed => parse_prefixed(pair),

        // bracketed expression block
        Rule::block => parse_block(pair),

        // keyworded composite expressions
        Rule::kw_function => parse_function(pair),
        Rule::kw_while => parse_while(pair),
        Rule::kw_for => parse_for(pair),
        Rule::kw_if_else => parse_if_else(pair),
        Rule::kw_repeat => parse_repeat(pair),
        Rule::kw_break => RExpr::Break,
        Rule::kw_continue => RExpr::Continue,

        // reserved values
        Rule::val_true => RExpr::Bool(true),
        Rule::val_false => RExpr::Bool(false),
        Rule::val_null => RExpr::Null,
        Rule::val_na => RExpr::NA,
        Rule::val_inf => RExpr::Inf,

        // reserved symbols
        Rule::ellipsis => RExpr::Ellipsis,

        // atomic values
        Rule::number => RExpr::Number(pair.as_str().parse::<f64>().unwrap()),
        Rule::integer => RExpr::Integer(pair.as_str().parse::<i32>().unwrap()),
        Rule::string => RExpr::String(String::from(pair.as_str())),

        // calls and symbols
        Rule::call => parse_call(pair),
        Rule::symbol_ident => parse_symbol(pair),
        Rule::symbol_backticked => RExpr::Symbol(String::from(pair.as_str())),

        // otherwise fail
        rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
    }
}

fn parse_block(pair: Pair<Rule>) -> RExpr {
    // extract each inline expression, and treat as unnamed list
    let exprs = pair
        .into_inner()
        .map(|i| parse_expr(i.into_inner()))
        .collect();

    // build call from symbol and list
    RExpr::new_primitive_call(RExprBlock, exprs)
}

fn parse_named(pair: Pair<Rule>) -> (Option<String>, RExpr) {
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    (Some(name), parse_expr(inner))
}

fn parse_list(pair: Pair<Rule>) -> RExprList {
    let exprs = pair
        .into_inner()
        .map(|i| match i.as_rule() {
            Rule::named => parse_named(i),
            Rule::ellipsis => (None, RExpr::Ellipsis),
            _ => (None, parse_primary(i)),
        })
        .collect();

    exprs
}

fn parse_call(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let name = String::from(inner.next().unwrap().as_str());
    RExpr::Call(
        Box::new(RExpr::String(name)),
        parse_list(inner.next().unwrap()),
    )
}

fn parse_function(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let params = parse_list(inner.next().unwrap());
    let body = parse_expr(inner);
    RExpr::Function(params, Box::new(body))
}

fn parse_if_else(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().unwrap().into_inner());
    let true_expr = parse_expr(inner.next().unwrap().into_inner());

    let false_expr = if let Some(false_block) = inner.next() {
        parse_expr(false_block.into_inner())
    } else {
        RExpr::Null
    };

    let args = RExprList::from(vec![cond, true_expr, false_expr]);
    RExpr::new_primitive_call(RExprIf, args)
}

fn parse_symbol(pair: Pair<Rule>) -> RExpr {
    RExpr::Symbol(String::from(pair.as_str()))
}

fn parse_for(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();

    let RExpr::Symbol(var) = parse_symbol(inner.next().unwrap()) else {
        unreachable!();
    };

    let iter = parse_expr(inner.next().unwrap().into_inner());
    let body = parse_expr(inner.next().unwrap().into_inner());

    let args = RExprList::from(vec![(Some(var), iter), (None, body)]);
    RExpr::new_primitive_call(RExprFor, args)
}

fn parse_while(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().unwrap().into_inner());
    let body = parse_expr(inner.next().unwrap().into_inner());
    let args = RExprList::from(vec![cond, body]);
    RExpr::new_primitive_call(RExprWhile, args)
}

fn parse_repeat(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let body = parse_expr(inner.next().unwrap().into_inner());
    let args = RExprList::from(vec![body]);
    RExpr::new_primitive_call(RExprRepeat, args)
}

fn parse_postfix(pair: Pair<Rule>) -> (RExpr, RExprList) {
    use RExpr::*;

    match pair.as_rule() {
        Rule::call => (Undefined, parse_list(pair)),
        Rule::index => {
            let args = parse_list(pair);
            (RExpr::as_primitive(PostfixIndex), args)
        }
        Rule::vector_index => (RExpr::as_primitive(PostfixVecIndex), parse_list(pair)),
        atom => unreachable!("invalid postfix operator '{:#?}'", atom),
    }
}

fn parse_postfixed(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner();
    let mut result = parse_primary(inner.next().unwrap());

    while let Some(next) = inner.next() {
        let (what, mut args) = parse_postfix(next);
        result = match what {
            // if postfix is parenthesized pairlist, it's a call to result
            RExpr::Undefined => RExpr::Call(Box::new(result), args),

            // otherwise call to a postfix operator with result as the first arg
            _ => {
                args.insert(0, result);

                RExpr::Call(Box::new(what), args)
            }
        };
    }

    result
}

fn parse_prefixed(pair: Pair<Rule>) -> RExpr {
    let mut inner = pair.into_inner().rev();
    let mut result = parse_postfixed(inner.next().unwrap());

    // iterate backwards through prefixes, applying prefixes from inside-out
    while let Some(prev) = inner.next() {
        result = match prev.as_rule() {
            Rule::subtract => {
                let args = RExprList::from(vec![result]);
                RExpr::new_primitive_call(PrefixSub, args)
            }
            _ => unreachable!("invalid prefix operator '{:#?}'", prev),
        }
    }

    result
}
