use r_derive::Primitive;

use crate::ast::*;
use crate::error::*;
use crate::lang::*;
use crate::vector::vectors::*;
use crate::callable::core::*;

#[derive(Debug, Clone, Primitive)]
pub struct PrimitivePaste;

impl PrimitiveSYM for PrimitivePaste {
    const SYM: &'static str = "paste";
}

impl Callable for PrimitivePaste {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let R::List(vals) = stack.parent_frame().eval_list_lazy(args)? else {
            unreachable!()
        };

        let mut vals = force_closures(vals, stack);

        let mut sep = String::from(" ");
        let mut should_collapse = false;
        let mut collapse = String::new();

        // find non-ellipsis arg indices
        let named_indices: Vec<usize> = vals
            .iter()
            .enumerate()
            .filter(|(_, (k, _))| *k == Some("collapse".to_string()) || *k == Some("sep".to_string()))
            .map(|(i, _)| i)
            .collect();

        // remove named sep and collapse args from our arguments and populate values
        for i in named_indices.iter().rev() {
            let (Some(k), v) = vals.remove(*i) else {
                continue
            };
            match (k.as_str(), v) {
                ("sep", R::Vector(Vector::Character(v))) => {
                    sep = v.get(0).unwrap().clone().to_string();
                }
                ("sep", _) => {
                    return Err(RSignal::Error(RError::Other(
                        "sep parameter must be a character value.".to_string(),
                    )));
                }
                ("collapse", R::Null) => continue,
                ("collapse", R::Vector(Vector::Character(v))) => {
                    should_collapse = true;
                    collapse = v.get(0).unwrap().clone().to_string();
                }
                ("collapse", _) => {
                    return Err(RSignal::Error(RError::WithCallStack(
                        Box::new(RError::Other("collapse parameter must be NULL or a character string.".to_string())),
                        stack.clone()
                    )))
                }
                _ => continue,
            }
        }

        // coerce all of our remaining arguments into vectors of strings
        let vec_s_vec: Vec<Vec<String>> = vals
            .into_iter()
            .map(|(_, v)| match v.as_character()? {
                R::Vector(v) => Ok(v.into()),
                _ => unreachable!(),
            })
            .collect::<Result<_, _>>()?;

        // maximum argument length, what we need to recycle the other args to meet
        let max_len = vec_s_vec.iter().map(|i| i.len()).max().unwrap_or(0);

        // NOTE: this _might_ be improved by initializing with String::with_capacity
        // as we should known the exact length of all the output strings, but would
        // have an overhead of pre-calculating sizes.
        let mut output = vec!["".to_string(); max_len];
        for i in 0..vec_s_vec.len() {
            for j in 0..output.len() {
                let vec_len = vec_s_vec[i].len();
                if i > 0 {
                    output[j].push_str(sep.as_str())
                };
                // Need to ignore null
                if vec_len == 0 {
                    continue;
                }
                output[j].push_str(vec_s_vec[i][j % vec_len].as_str())
            }
        }

        if should_collapse {
            output = vec![output.join(&collapse)];
        }

        Ok(R::Vector(output.into()))
    }
}

#[cfg(test)]
mod test_primitive_paste {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_primitive_paste_01() {
        let mut stack = CallStack::new();
        let expr = parse("paste(1, 2, collapse = NULL)").unwrap();
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<_> = vec!["1 2"];
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_02() {
        let mut stack = CallStack::new();
        let expr = parse("paste(null)").unwrap();
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<&str> = vec![];
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_03() {
        let mut stack = CallStack::new();
        let expr = parse("paste(1.1, null, 2, false, 'a', c(1.0, 2.0), sep = '+')").unwrap();
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<_> = vec!["1.1++2+false+a+1".to_string(), "1.1++2+false+a+2".to_string()];
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_04() {
        let mut stack = CallStack::new();
        let expr = parse("paste(1.1, null, 2, false, 'a', c(1.0, 2.0))").unwrap();
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<_> = vec!["1.1  2 false a 1".to_string(), "1.1  2 false a 2".to_string()];
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_05() {
        let mut stack = CallStack::new();
        let expr = parse("paste(c(1, 2, 3, 4, 5), c('st', 'nd', 'rd', c('th', 'th')), sep = '')").unwrap(); 
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<_> = vec!["1st".to_string(), "2nd".to_string(), "3rd".to_string(), "4th".to_string(), "5th".to_string()];
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_06() {
        let mut stack = CallStack::new();
        let expr = parse("paste(1.1, null, 2, false, 'a', c(1.0, 2.0), , collapse = '+')").unwrap();
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<_> = vec!["1.1  2 false a 1+1.1  2 false a 2".to_string()];
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_07() {
        let mut stack = CallStack::new();
        let expr = parse("paste(1, 2, 3, collapse = '+')").unwrap();
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<_> = vec!["1 2 3".to_string()];
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_08() {
        let mut stack = CallStack::new();
        let expr = parse("paste(c(1, 2), 3, 4, 5, sep = '-', collapse = '+')").unwrap();
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<_> = vec!["1-3-4-5+2-3-4-5".to_string()];
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_09() {
        let mut stack = CallStack::new();
        let x_val = stack.eval(parse("\"<collapse>\"").unwrap()).unwrap();
        stack.last_frame().env.insert("x".to_string(), x_val);
        let expr = parse("paste(c('a', 'b'), collapse = x)").unwrap();
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<_> = vec!["a<collapse>b".to_string()];
        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_10() {
        let mut stack = CallStack::new();
        let expr = parse("paste('a', c('b', 'c'), collapse = '')").unwrap();
        let R::Vector(res) = stack.eval(expr).unwrap() else { unimplemented!() };
        let observed: Vec<_> = res.into();
        let expected: Vec<_> = vec!["a ba c".to_string()];
        assert_eq!(observed, expected);
    }
}