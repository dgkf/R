use std::rc::Rc;

use crate::lang::{EvalResult, Signal};
use crate::object::types::Character;
use crate::object::*;
use crate::{error::*, internal_err};

pub trait Context: std::fmt::Debug + std::fmt::Display {
    #[inline]
    fn get(&mut self, name: String) -> EvalResult {
        (*self).env().get(name)
    }
    #[inline]
    fn get_mut(&mut self, name: String) -> EvalResult {
        self.get(name)
    }

    #[inline]
    fn get_ellipsis(&mut self) -> EvalResult {
        let err = Err(Signal::Error(Error::IncorrectContext("...".to_string())));
        self.get("...".to_string()).or(err)
    }

    #[inline]
    fn assign_lazy(&mut self, _to: Expr, _from: Expr) -> EvalResult {
        Err(Signal::Error(Error::IncorrectContext("<-".to_string())))
    }

    #[inline]
    fn assign(&mut self, _to: Expr, _from: Obj) -> EvalResult {
        Err(Signal::Error(Error::IncorrectContext("<-".to_string())))
    }

    fn env(&self) -> Rc<Environment>;

    #[inline]
    fn eval_call(&mut self, expr: Expr) -> EvalResult {
        self.eval(expr)
    }
    #[inline]
    fn eval_call_mut(&mut self, expr: Expr) -> EvalResult {
        Error::CannotEvaluateAsMutable(expr.clone()).into()
    }

    #[inline]
    fn eval(&mut self, expr: Expr) -> EvalResult {
        self.env().eval(expr)
    }

    #[inline]
    fn eval_mut(&mut self, expr: Expr) -> EvalResult {
        Error::CannotEvaluateAsMutable(expr.clone()).into()
    }

    #[inline]
    fn eval_in(&mut self, expr: Expr, mut env: Rc<Environment>) -> EvalResult {
        env.eval(expr)
    }

    #[inline]
    fn eval_and_finalize(&mut self, expr: Expr) -> EvalResult {
        self.eval(expr)
    }

    #[inline]
    fn eval_binary(&mut self, exprs: (Expr, Expr)) -> Result<(Obj, Obj), Signal> {
        Ok((
            self.eval_and_finalize(exprs.0)?,
            self.eval_and_finalize(exprs.1)?,
        ))
    }

    fn eval_list_lazy(&mut self, l: ExprList) -> EvalResult {
        Ok(Obj::List(List::from(
            l.into_iter()
                .map(|pair| match pair {
                    (_, Expr::Ellipsis(None)) => {
                        if let Ok(Obj::List(ellipsis)) = self.get_ellipsis() {
                            Ok(ellipsis.iter_pairs())
                        } else {
                            Ok(List::new().iter_pairs())
                        }
                    }
                    (_, Expr::Ellipsis(Some(name))) => {
                        if let Ok(Obj::List(more)) = self.get(name) {
                            Ok(more.iter_pairs())
                        } else {
                            internal_err!()
                        }
                    }
                    // Avoid creating a new closure just to point to another, just reuse it
                    (k, Expr::Symbol(s)) => match self.env().get(s.clone()) {
                        Ok(c @ Obj::Promise(..)) => {
                            let k = k.map_or(OptionNA::NA, OptionNA::Some);
                            Ok(List::from(vec![(k, c)]).iter_pairs())
                        }
                        _ => {
                            let k = k.map_or(OptionNA::NA, OptionNA::Some);
                            Ok(List::from(vec![(
                                k,
                                Obj::Promise(None, Expr::Symbol(s), self.env()),
                            )])
                            .iter_pairs())
                        }
                    },
                    (k, c @ Expr::Call(..)) => {
                        let k = k.map_or(OptionNA::NA, OptionNA::Some);
                        let elem = vec![(k, Obj::Promise(None, c, self.env()))];
                        Ok(List::from(elem).iter_pairs())
                    }
                    (k, v) => {
                        let k = k.map_or(OptionNA::NA, OptionNA::Some);
                        if let Ok(elem) = self.eval(v) {
                            Ok(List::from(vec![(k, elem)]).iter_pairs())
                        } else {
                            internal_err!()
                        }
                    }
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        )))
    }

    fn eval_list_eager(&mut self, l: ExprList) -> EvalResult {
        Ok(Obj::List(List::from(
            l.into_iter()
                .map(|pair| match pair {
                    (_, Expr::Ellipsis(None)) => {
                        if let Ok(Obj::List(ellipsis)) = self.get_ellipsis() {
                            Ok(ellipsis.iter_pairs())
                        } else {
                            Ok(List::from(Vec::<(Character, Obj)>::new()).iter_pairs())
                        }
                    }
                    (_, Expr::Ellipsis(Some(name))) => {
                        if let Ok(Obj::List(more)) = self.get(name) {
                            Ok(more.iter_pairs())
                        } else {
                            Ok(List::from(Vec::<(Character, Obj)>::new()).iter_pairs())
                        }
                    }
                    (k, v) => match self.eval_and_finalize(v) {
                        Ok(elem) => {
                            let k = k.map_or(OptionNA::NA, OptionNA::Some);
                            Ok(List::from(vec![(k, elem)]).iter_pairs())
                        }
                        Err(e) => Err(e),
                    },
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        )))
    }
}
