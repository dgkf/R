use r_derive::Primitive;

use crate::ast::*;
use crate::error::*;
use crate::lang::*;
use crate::vector::vectors::*;
use crate::callable::core::*;

#[derive(Debug, Clone, Primitive, PartialEq)]
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
mod test {
    use super::*;
    use crate::r;

    #[test]
    fn numeric_input() {
        assert_eq!(
            r!{ paste(1, 2, collapse = NULL) }, 
            r!{ "1 2" }
        );
    }

    #[test]
    fn only_null() {
        assert_eq!(
            r!{ paste(null) }, 
            Ok(R::Vector(Vector::Character(vec![])))
        );
    }

    #[test]
    fn ignore_null() {
        assert_eq!(
            r!{ paste(1.1, null, 2, false, "a", c(1.0, 2.0)) },
            r!{ c("1.1  2 false a 1", "1.1  2 false a 2") }
        )
    }

    #[test]
    fn sep_param() {
        assert_eq!(
            r!{ paste(1.1, null, 2, false, "a", c(1.0, 2.0), sep = "+") },
            r!{ c("1.1++2+false+a+1", "1.1++2+false+a+2") }
        )
    }

    #[test]
    fn param_recycling() {
        assert_eq!(
            r!{ paste(c(1, 2, 3, 4, 5), c("st", "nd", "rd", c("th", "th")), sep = "") },
            r!{ c("1st", "2nd", "3rd", "4th", "5th") }
        )
    }

    #[test]
    fn collapse_param() {
        assert_eq!(
            r!{ paste(1.1, null, 2, false, "a", c(1.0, 2.0), , collapse = "+") },
            r!{ "1.1  2 false a 1+1.1  2 false a 2" }
        )
    }

    #[test]
    fn non_vec_collapse() {
        assert_eq!(
            r!{ paste(1, 2, 3, collapse = "+") },
            r!{ "1 2 3" }
        )
    }

    #[test]
    fn collapse_and_sep() {
        assert_eq!(
            r!{ paste(c(1, 2), 3, 4, 5, sep = "-", collapse = "+") },
            r!{ "1-3-4-5+2-3-4-5" }
        )
    }

    #[test]
    fn param_from_parent_frame() {
        assert_eq!(
            r!{ x <- "<collapse>"; paste(c("a", "b"), collapse = x) },
            r!{ "a<collapse>b" }
        )
    }

    #[test]
    fn collapse_empty_string() {
        assert_eq!(
            r!{ paste("a", c("b", "c"), collapse = "") },
            r!{ "a ba c" }
        )
    }
}
