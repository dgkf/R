use crate::ast::*;
use crate::error::*;
use crate::lang::*;
use crate::r_builtins::builtins::force_closures;
use crate::r_vector::vectors::*;

pub fn primitive_paste(args: ExprList, stack: &mut CallStack) -> EvalResult {
    let R::List(vals) = stack.eval_list(args)? else {
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

#[cfg(test)]
mod test_primitive_paste {
    use super::*;
    use crate::parser::parse_args;

    #[test]
    fn test_primitive_paste_01() {
        let mut env = Environment::default();
        let args = parse_args("paste(1, 2, collapse = NULL)").unwrap();
        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<_> = vec!["1 2"];

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_02() {
        let mut env = Environment::default();
        let args = parse_args("paste(null)").unwrap();
        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<&str> = vec![];

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_03() {
        let mut env = Environment::default();
        let args = parse_args("paste(1.1, null, 2, false, 'a', c(1.0, 2.0), sep = '+')").unwrap();
        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<_> = vec!["1.1++2+false+a+1", "1.1++2+false+a+2"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_04() {
        let mut env = Environment::default();
        let args = parse_args("paste(1.1, null, 2, false, 'a', c(1.0, 2.0))").unwrap();
        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<_> = vec!["1.1  2 false a 1", "1.1  2 false a 2"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_05() {
        let mut env = Environment::default();
        let args =
            parse_args("paste(c(1, 2, 3, 4, 5), c('st', 'nd', 'rd', c('th', 'th')), sep = '')")
                .unwrap();
        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<_> = vec!["1st", "2nd", "3rd", "4th", "5th"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_06() {
        let mut env = Environment::default();
        let args =
            parse_args("paste(1.1, null, 2, false, 'a', c(1.0, 2.0), , collapse = '+')").unwrap();
        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<_> = vec!["1.1  2 false a 1+1.1  2 false a 2"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_07() {
        let mut env = Environment::default();
        let args = parse_args("paste(1, 2, 3, collapse = '+')").unwrap();
        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<_> = vec!["1 2 3"].iter().map(|s| s.to_string()).collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_08() {
        let mut env = Environment::default();
        let args = parse_args("paste(c(1, 2), 3, 4, 5, sep = '-', collapse = '+')").unwrap();
        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<_> = vec!["1-3-4-5+2-3-4-5"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_09() {
        let mut env = Environment::default();
        // Passing x vector to the environment so that paste can use it
        env.insert(
            "x".to_string(),
            R::Vector(Vector::Character(vec![OptionNA::Some(
                "<collapse>".to_string(),
            )])),
        );
        let args = parse_args("paste(c('a', 'b'), collapse = x)").unwrap();

        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<_> = vec!["a<collapse>b"].iter().map(|s| s.to_string()).collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_10() {
        let mut env = Environment::default();
        let args = parse_args("paste('a', c('b', 'c'), collapse = '')").unwrap();

        let R::Vector(observed) = primitive_paste(args, &mut env).unwrap() else {unimplemented!()};
        let observed: Vec<_> = observed.into();
        let expected: Vec<_> = vec!["a ba c"].iter().map(|s| s.to_string()).collect();

        assert_eq!(observed, expected);
    }
}
