use crate::callable::core::*;
use crate::error::*;
use crate::formals;
use crate::lang::*;
use crate::object::types::Character;
use crate::object::*;
use r_derive::*;

/// Paste Objects into Strings
///
/// Construct strings out of objects, producing a `character` `vector`
/// from element-wise inputs and optionally collapsing results into
/// a single `string`
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// paste(..., sep = " ", collapse = NULL)
/// ```
///
/// ## Arguments
///
/// `...`: Objects to paste into strings.
/// `sep`: A separator to insert when pasting.
/// `collapse`: An optional string used to concatenate all elements
///   of the pasted `character` `vector`.
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// paste(1:3, 4:6)
/// ```
///
/// ```custom,{class=r-repl}
/// paste(1:3, 4:6, sep = "-")
/// ```
///
/// ```custom,{class=r-repl}
/// paste(1:3, 4:6, sep = "-", collapse = ":")
/// ```
///
#[doc(alias = "paste")]
#[builtin(sym = "paste")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitivePaste;

formals!(PrimitivePaste, "(..., sep = ' ', collapse = null)");

impl Callable for PrimitivePaste {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (args, ellipsis) = self.match_arg_exprs(args, stack)?;

        let ellipsis = force_promises(ellipsis, stack)?;

        let args = force_promises(args, stack)?;

        let mut sep = String::from(" ");
        let mut should_collapse = false;
        let mut collapse = String::new();

        // remove named sep and collapse args from our arguments and populate values
        for (k, v) in args.iter().rev() {
            let Character::Some(k) = k else { continue };

            match (k.as_str(), v) {
                ("sep", Obj::Vector(v)) => {
                    sep = (*v).clone().into();
                }
                ("sep", _) => {
                    return Err(Signal::Error(Error::Other(
                        "sep parameter must be a character value.".to_string(),
                    )));
                }
                ("collapse", Obj::Null) => continue,
                ("collapse", Obj::Vector(v)) => {
                    should_collapse = true;
                    collapse = (*v).clone().into();
                }
                ("collapse", _) => {
                    return Err(Signal::Error(Error::WithCallStack(
                        Box::new(Error::Other(
                            "collapse parameter must be NULL or a character string.".to_string(),
                        )),
                        stack.clone(),
                    )))
                }
                _ => continue,
            }
        }

        // coerce all of our remaining arguments into vectors of strings
        let vec_s_vec: Vec<Vec<String>> = ellipsis
            .into_iter()
            .map(|(_, v)| -> Result<Vec<String>, Signal> {
                match v.as_character()? {
                    Obj::Vector(v) => Ok(v.into()),
                    _ => unreachable!(),
                }
            })
            .collect::<Result<_, _>>()?;

        // maximum argument length, what we need to recycle the other args to meet
        let max_len = vec_s_vec.iter().map(|i| i.len()).max().unwrap_or(0);

        // NOTE: this _might_ be improved by initializing with String::with_capacity
        // as we should known the exact length of all the output strings, but would
        // have an overhead of pre-calculating sizes.
        let mut output = vec!["".to_string(); max_len];
        (0..vec_s_vec.len()).for_each(|i| {
            (0..output.len()).for_each(|j| {
                let vec_len = vec_s_vec[i].len();
                if i > 0 {
                    output[j].push_str(sep.as_str())
                };
                // Need to ignore null
                if vec_len != 0 {
                    output[j].push_str(vec_s_vec[i][j % vec_len].as_str())
                }
            });
        });

        if should_collapse {
            output = vec![output.join(&collapse)];
        }
        Ok(Obj::Vector(output.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::r;

    #[test]
    fn numeric_input() {
        assert_eq!(r! { paste(1, 2, collapse = NULL) }, r! { "1 2" });
    }

    #[test]
    fn only_null() {
        assert_eq!(
            r! { paste(null) },
            Ok(Obj::Vector(Vector::from(Vec::<Character>::new())))
        );
    }

    #[test]
    fn ignore_null() {
        assert_eq!(
            r! { paste(1.1, null, 2, false, "a", c(1.0, 2.0)) },
            r! { c("1.1  2 false a 1", "1.1  2 false a 2") }
        )
    }

    #[test]
    fn sep_param() {
        assert_eq!(
            r! { paste(1.1, null, 2, false, "a", c(1.0, 2.0), sep = "+") },
            r! { c("1.1++2+false+a+1", "1.1++2+false+a+2") }
        )
    }

    #[test]
    fn param_recycling() {
        assert_eq!(
            r! { paste(c(1, 2, 3, 4, 5), c("st", "nd", "rd", c("th", "th")), sep = "") },
            r! { c("1st", "2nd", "3rd", "4th", "5th") }
        )
    }

    #[test]
    fn collapse_param() {
        assert_eq!(
            r! { paste(1.1, null, 2, false, "a", c(1.0, 2.0), collapse = "+") },
            r! { "1.1  2 false a 1+1.1  2 false a 2" }
        )
    }

    #[test]
    fn non_vec_collapse() {
        assert_eq!(r! { paste(1, 2, 3, collapse = "+") }, r! { "1 2 3" })
    }

    #[test]
    fn collapse_and_sep() {
        assert_eq!(
            r! { paste(c(1, 2), 3, 4, 5, sep = "-", collapse = "+") },
            r! { "1-3-4-5+2-3-4-5" }
        )
    }

    #[test]
    fn param_from_parent_frame() {
        assert_eq!(
            r! { x <- "<collapse>"; paste(c("a", "b"), collapse = x) },
            r! { "a<collapse>b" }
        )
    }

    #[test]
    fn collapse_empty_string() {
        assert_eq!(
            r! { paste("a", c("b", "c"), collapse = "") },
            r! { "a ba c" }
        )
    }
}
