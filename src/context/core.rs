use std::rc::Rc;

use crate::lang::{EvalResult, Signal};
use crate::error::*;
use crate::object::*;

pub trait Context: std::fmt::Debug + std::fmt::Display {
    fn get(&mut self, name: String) -> EvalResult {
        (*self).env().get(name)
    }

    fn get_ellipsis(&mut self) -> EvalResult {
        let err = Err(Signal::Error(RError::IncorrectContext("...".to_string())));
        self.get("...".to_string()).or(err)
    }

    fn assign_lazy(&mut self, _to: Expr, _from: Expr) -> EvalResult {
        Err(Signal::Error(RError::IncorrectContext("<-".to_string())))
    }

    fn assign(&mut self, _to: Expr, _from: Obj) -> EvalResult {
        Err(Signal::Error(RError::IncorrectContext("<-".to_string())))
    }

    fn env(&self) -> Rc<Environment>;

    fn eval_call(&mut self, expr: Expr) -> EvalResult {
        self.eval(expr)
    }

    fn eval(&mut self, expr: Expr) -> EvalResult {
        self.env().eval(expr)
    }

    fn eval_and_finalize(&mut self, expr: Expr) -> EvalResult {
        self.eval(expr)
    }

    fn eval_binary(&mut self, exprs: (Expr, Expr)) -> Result<(Obj, Obj), Signal> {
        Ok((self.eval(exprs.0)?, self.eval(exprs.1)?))
    }

    fn eval_list_lazy(&mut self, l: ExprList) -> EvalResult {
        Ok(Obj::List(List::from(
            l.into_iter()
                .flat_map(|pair| match pair {
                    (_, Expr::Ellipsis(None)) => {
                        if let Ok(Obj::List(ellipsis)) = self.get_ellipsis() {
                            ellipsis.values.borrow_mut().clone().into_iter()
                        } else {
                            vec![].into_iter()
                        }
                    }
                    (k, e @ (Expr::Call(..) | Expr::Symbol(..))) => {
                        let elem = vec![(k, Obj::Closure(e, self.env()))];
                        elem.into_iter()
                    }
                    (k, v) => {
                        if let Ok(elem) = self.eval(v) {
                            vec![(k, elem)].into_iter()
                        } else {
                            unreachable!()
                        }
                    }
                })
                .collect::<Vec<_>>(),
        )))
    }

    fn eval_list_eager(&mut self, l: ExprList) -> EvalResult {
        Ok(Obj::List(List::from(
            l.into_iter()
                .map(|pair| match pair {
                    (_, Expr::Ellipsis(None)) => {
                        if let Ok(Obj::List(ellipsis)) = self.get_ellipsis() {
                            Ok(ellipsis.values.borrow_mut().clone().into_iter())
                        } else {
                            Ok(vec![].into_iter())
                        }
                    }
                    (k, v) => match self.eval(v) {
                        Ok(elem) => Ok(vec![(k, elem)].into_iter()),
                        Err(e) => Err(e),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        )))
    }
}
