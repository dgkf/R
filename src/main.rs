#[macro_use]
extern crate pest_derive;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Display, Error, Formatter};
use std::io::{self, stdout, BufRead, Write};
use std::rc::Rc;

use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest::Parser;

#[derive(Debug)]
pub enum RError {
    VariableNotFound(String),
}

impl RError {
    fn as_str(&self) -> String {
        match self {
            RError::VariableNotFound(v) => format!("Variable '{}' not found", v.as_str()),
        }
    }
}

impl fmt::Display for RError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RParser;

#[derive(Debug, Clone)]
pub enum RExpr {
    Null,
    Bool(bool),
    Number(f32),
    Integer(i32),
    String(String),
    Symbol(String),
    List(RExprList),
    Function(RExprList, Box<RExpr>), // TODO: capture environment
    Call(Box<dyn Callable>, RExprList),
}

impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.callable_as_str())
    }
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Self {
        self.callable_clone()
    }
}

// internal callables
#[derive(Debug, Clone)]
pub struct Name(String);

#[derive(Debug, Clone)]
pub struct RExprBlock;

#[derive(Debug, Clone)]
pub struct InfixAdd;

#[derive(Debug, Clone)]
pub struct InfixAssign;

pub trait Callable {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError>;
    fn callable_clone(&self) -> Box<dyn Callable>;
    fn callable_as_str(&self) -> &str;
}

impl Callable for RExprBlock {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        let mut value = Ok(R::Null);
        for expr in args.values {
            let result = eval(expr, env);
            match result {
                Ok(_) => value = result,
                _ => return result,
            }
        }
        value
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "{"
    }
}

impl Callable for InfixAssign {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        if let RExpr::Symbol(s) = &args.values[0] {
            let value = eval(args.values[1].clone(), env)?;
            env.insert(&s, value.clone());
            Ok(value)
        } else {
            unimplemented!()
        }
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "<-"
    }
}

impl Callable for InfixAdd {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        let lhs = eval(args.values[0].clone(), env)?;
        let rhs = eval(args.values[1].clone(), env)?;
        let op = |l, r| l + r;

        // TODO: improve vector type unification prior to math operations
        let res = match (lhs, rhs) {
            (R::Numeric(e1), R::Numeric(e2)) => R::Numeric(op_vectorized_recycled(op, e1, e2)),
            (R::Numeric(e1), R::Integer(e2)) => {
                if let R::Numeric(e2) = R::Integer(e2).as_numeric() {
                    R::Numeric(op_vectorized_recycled(op, e1, e2))
                } else {
                    R::Null
                }
            }
            (R::Integer(e1), R::Numeric(e2)) => {
                if let R::Numeric(e1) = R::Integer(e1).as_numeric() {
                    R::Numeric(op_vectorized_recycled(op, e1, e2))
                } else {
                    R::Null
                }
            }
            (R::Integer(e1), R::Integer(e2)) => {
                R::Integer(op_vectorized_recycled(|l, r| l + r, e1, e2))
            }
            _ => R::Null,
        };

        Ok(res)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "+"
    }
}

pub fn match_args(formals: RExprList, args: RExprList) -> RExprList {
    use RExprListKey::*;

    let mut assigned = vec![false; formals.values.len()];
    let mut matched_args = formals.clone();

    // assign named args
    for (k, v) in args.keys.iter().zip(args.values.iter()) {
        if let Some(argname) = k {
            let key = Some(Name(argname.clone()));
            let index = matched_args.insert(key, v.clone());
            if index >= assigned.len() {
                assigned.extend(vec![false; index - assigned.len() + 1])
            }
            assigned[index] = true;
        }
    }

    // backfill unnamed args
    for (k, v) in args.keys.iter().zip(args.values.iter()) {
        if let None = k {
            if let Some(next_index) = assigned.iter().position(|&i| !i) {
                let key = if next_index < formals.keys.len() {
                    Some(Name(formals.keys[next_index].clone().unwrap()))
                } else {
                    Some(Index(next_index))
                };
                let index = matched_args.insert(key, v.clone());
                if index >= assigned.len() {
                    assigned.extend(vec![false; index - assigned.len() + 1])
                }
                assigned[index] = true;
            } else {
                let key = Some(Index(matched_args.values.len()));
                let index = matched_args.insert(key, v.clone());
                if index >= assigned.len() {
                    assigned.extend(vec![false; index - assigned.len() + 1])
                }
                assigned[index] = true;
            }
        }
    }

    matched_args
}

impl Callable for String {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        if let R::Function(formals, body) = env.get(self.clone())? {
            // set up our local scope, a child environment of calling environment
            let local_scope = Environment::new(Env {
                parent: Some(Rc::clone(env)),
                ..Default::default()
            });

            // match arguments against function signature
            let args = match_args(formals, args);

            // create promises for matched args, do not evaluate until used
            for (k, expr) in args.keys.iter().zip(args.values.iter()) {
                if let Some(formal) = k {
                    local_scope.insert(&formal, R::Closure(expr.clone(), Rc::clone(env)));
                }
            }

            // evaluate body in local scope
            eval(body, &mut Rc::clone(&local_scope))
        } else {
            unimplemented!();
        }
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RExprList {
    keys: Vec<Option<String>>,
    values: Vec<RExpr>,
}

#[derive(Debug, Clone)]
pub enum RExprListKey {
    Name(String),
    Index(usize),
}

impl RExprList {
    fn get(&self, key: RExprListKey) -> Option<RExpr> {
        use RExprListKey::*;
        match key {
            Name(s) => {
                if let Some(index) = self.keys.iter().position(|i| i == &Some(s.clone())) {
                    Some(self.values[index].clone())
                } else {
                    None
                }
            }
            Index(i) => Some(self.values[i].clone()),
        }
    }

    fn insert(&mut self, key: Option<RExprListKey>, value: RExpr) -> usize {
        use RExprListKey::*;
        match key {
            Some(Name(s)) => {
                if let Some(index) = self.keys.iter().position(|i| i == &Some(s.clone())) {
                    self.values[index] = value;
                    index
                } else {
                    self.keys.push(Some(s));
                    self.values.push(value);
                    self.values.len()
                }
            }
            Some(Index(index)) => {
                if index < self.values.len() {
                    self.values[index] = value;
                    index
                } else {
                    let n = index - self.values.len();
                    self.keys.extend(vec![None; n]);
                    self.values.extend(vec![RExpr::Null; n - 1]);
                    self.values.push(value);
                    self.values.len()
                }
            }
            None => {
                self.keys.push(None);
                self.values.push(value);
                self.values.len()
            }
        }
    }
}

impl From<Vec<RExpr>> for RExprList {
    fn from(values: Vec<RExpr>) -> Self {
        let mut l = RExprList {
            ..Default::default()
        };
        for value in values {
            l.insert(None, value);
        }
        l
    }
}

impl From<Vec<(Option<String>, RExpr)>> for RExprList {
    fn from(values: Vec<(Option<String>, RExpr)>) -> Self {
        let mut l = RExprList {
            ..Default::default()
        };
        for (key, value) in values {
            match key {
                Some(s) => l.insert(Some(RExprListKey::Name(s)), value),
                None => l.insert(None, value),
            };
        }
        l
    }
}

impl FromIterator<RExpr> for RExprList {
    fn from_iter<T: IntoIterator<Item = RExpr>>(iter: T) -> Self {
        let mut l = RExprList {
            ..Default::default()
        };
        for value in iter {
            l.insert(None, value);
        }
        l
    }
}

impl FromIterator<(Option<String>, RExpr)> for RExprList {
    fn from_iter<T: IntoIterator<Item = (Option<String>, RExpr)>>(iter: T) -> Self {
        let mut l = RExprList {
            ..Default::default()
        };
        for (key, value) in iter {
            match key {
                Some(s) => l.insert(Some(RExprListKey::Name(s)), value),
                None => l.insert(None, value),
            };
        }
        l
    }
}

#[derive(Debug, Clone)]
pub enum R {
    Null,
    Logical(Logical),
    Numeric(Numeric),
    Integer(Integer),
    Character(Character),
    List(List),

    Expr(RExpr),
    Closure(RExpr, Environment),
    Function(RExprList, RExpr),
    Environment(Rc<Environment>),
}

pub type Logical = Vec<bool>;
pub type Numeric = Vec<f32>;
pub type Integer = Vec<i32>;
pub type Character = Vec<String>;
pub type List = Vec<(Option<String>, R)>;

pub type Environment = Rc<Env>;

#[derive(Debug, Default, Clone)]
pub struct Env {
    values: RefCell<HashMap<String, R>>,
    parent: Option<Environment>,
}

impl Env {
    pub fn get(&self, name: String) -> Result<R, RError> {
        if let Some(value) = self.values.borrow().get(&name) {
            let result = value.clone();
            return match result {
                R::Closure(expr, env) => eval(expr, &mut Rc::clone(&env)),
                _ => Ok(result),
            };
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            Err(RError::VariableNotFound(name))
        }
    }

    pub fn insert(&self, name: &String, value: R) {
        self.values.borrow_mut().insert(name.clone(), value);
    }
}

impl R {
    pub fn as_integer(self) -> R {
        match self {
            R::Integer(_) => self,
            R::Numeric(v) => R::Integer(v.iter().map(|&i| i as i32).collect()),
            atom => unreachable!("{:?} cannot be coerced to integer", atom),
        }
    }
    pub fn as_numeric(self) -> R {
        match self {
            R::Numeric(_) => self,
            R::Integer(v) => R::Numeric(v.iter().map(|&i| i as f32).collect()),
            atom => unreachable!("{:?} cannot be coerced to numeric", atom),
        }
    }
}

pub fn op_vectorized_recycled<F, T>(f: F, mut e1: Vec<T>, e2: Vec<T>) -> Vec<T>
where
    F: Fn(T, T) -> T,
    T: Clone + Display,
{
    if e2.len() > e1.len() {
        return op_vectorized_recycled(f, e2, e1);
    }

    for i in 0..e1.len() {
        e1[i] = f(e1[i].clone(), e2[i % e2.len()].clone())
    }

    e1
}

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
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
            Rule::symbol_backticked => (
                Some(String::from(i.to_string())),
                RExpr::Symbol(String::from(i.as_str())),
            ),
            Rule::symbol_ident => (
                Some(String::from(i.as_str())),
                RExpr::Symbol(String::from(i.as_str())),
            ),
            Rule::expr | Rule::inline | Rule::block => (None, parse_expr(i.into_inner())),
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

pub fn parse_expr(pairs: Pairs<Rule>) -> RExpr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::kw_function => parse_function(primary),
            Rule::kw_while => unimplemented!(),
            Rule::kw_for => unimplemented!(),
            Rule::kw_if_else => unimplemented!(),
            Rule::kw_repeat => unimplemented!(),
            Rule::call => parse_call(primary),
            Rule::expr => parse_expr(primary.into_inner()),
            Rule::inline => parse_expr(primary.into_inner()),
            Rule::block => parse_block(primary),
            Rule::list => RExpr::List(parse_list(primary)),
            Rule::boolean_true => RExpr::Bool(true),
            Rule::boolean_false => RExpr::Bool(false),
            Rule::number => RExpr::Number(primary.as_str().parse::<f32>().unwrap()),
            Rule::integer => RExpr::Integer(primary.as_str().parse::<i32>().unwrap()),
            Rule::string_expr => parse_expr(primary.into_inner()), // TODO: improve grammar to avoid unnecessary parse
            Rule::string => RExpr::String(String::from(primary.as_str())),
            Rule::null => RExpr::Null,
            Rule::symbol_ident => RExpr::Symbol(String::from(primary.as_str())),
            Rule::symbol_backticked => RExpr::Symbol(String::from(primary.as_str())),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            // infix operator with two unnamed arguments
            let args = vec![(None, lhs), (None, rhs)].into();

            let op: Box<dyn Callable> = match op.as_rule() {
                Rule::add => Box::new(InfixAdd),
                Rule::subtract => Box::new("-".to_string()),
                Rule::multiply => Box::new("*".to_string()),
                Rule::divide => Box::new("/".to_string()),
                Rule::assign => Box::new(InfixAssign),
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };

            RExpr::Call(op, args)
        })
        .parse(pairs)
}

pub fn eval(expr: RExpr, env: &mut Environment) -> Result<R, RError> {
    match expr {
        RExpr::Null => Ok(R::Null),
        RExpr::Number(x) => Ok(R::Numeric(vec![x])),
        RExpr::Integer(x) => Ok(R::Integer(vec![x])),
        RExpr::Bool(x) => Ok(R::Logical(vec![x])),
        RExpr::String(x) => Ok(R::Character(vec![x])),
        RExpr::Function(formals, body) => Ok(R::Function(formals, *body)),
        RExpr::Call(what, list) => Ok(what.call(list, env)?),
        RExpr::Symbol(name) => env.get(name),
        _ => unimplemented!(),
    }
}

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock().lines();
    let global_env = Environment::default();

    loop {
        print!("> ");
        stdout().flush().unwrap();

        let line = stdin.next().unwrap();
        match RParser::parse(Rule::expr, &line?) {
            Ok(mut pairs) => {
                let inner = pairs.next().unwrap().into_inner();
                let parsed_expr = parse_expr(inner);
                // println!("Parsed: {:#?}", parsed_expr);
                let result = eval(parsed_expr, &mut Rc::clone(&global_env));
                match result {
                    Ok(val) => println!("{:?}", val),
                    Err(e) => println!("{}", e),
                }
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }
}
