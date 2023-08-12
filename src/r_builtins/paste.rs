use crate::ast::*;
use crate::error::*;
use crate::lang::*;
use crate::r_vector::vectors::*;

pub fn primitive_paste(args: ExprList, env: &mut Environment) -> EvalResult {
    let R::List(vals) = env.eval_list(args)? else {
        unreachable!()
    };

    // Force any closures that were created during call. This helps with using
    // variables as argument for sep and collapse parameters.
    let vals: Vec<_> = vals
        .into_iter()
        .map(|(k, v)| (k, v.clone().force().unwrap_or(R::Null))) // TODO: raise this error
        .collect();

    let mut char_vals: Vec<_> = vec![];
    let mut sep = " ".to_string();
    let mut collapse = String::new();

    // Extract sep and collapse parameters, leave the rest for processing
    for (k, v) in vals {
        let k_clone = k.clone().unwrap_or("".to_string());

        match k_clone.as_str() {
            "sep" => {
                sep = match v {
                    // We need to check whether the supplied sep value is a
                    // character. R does not accept non-character values. If
                    // we use R::Vector(v) => R::Vector(v.as_character()) then
                    // this will coerce non-valid sep value to a character
                    R::Vector(Vector::Character(v)) => v.get(0).unwrap().clone().to_string(),
                    _ => {
                        return Err(RSignal::Error(RError::Other(
                            "sep parameter must be a character string!".to_string(),
                        )))
                    }
                }
            }
            "collapse" => {
                collapse = match v {
                    R::Null => continue,
                    R::Vector(Vector::Character(v)) => v.get(0).unwrap().clone().to_string(),
                    _ => {
                        return Err(RSignal::Error(RError::Other(
                            "collapse parameter must be NULL or a character string!".to_string(),
                        )))
                    }
                }
            }
            _ => {
                if let R::List(_) = v {
                    return Err(RSignal::Error(RError::Other(
                        "list is not supported in paste() yet!".to_string(),
                    )));
                }
                // Leave the rest for processing. Coerce everything into character.
                char_vals.push(v.clone().as_character())
            }
        }
    }

    let vec_of_vectors: Vec<_> = char_vals
        .iter()
        .map(|v| v.as_ref().unwrap().get_vec_string())
        // Filtering out Null values
        .filter(|v| v.len() != 0)
        .collect();

    // Need the length of longest vector to create an empty vector that others
    // will go through and re-cycle values as needed
    let n = vec_of_vectors
        .iter()
        .max_by_key(|x| x.len())
        .unwrap_or(&vec![])
        .len();

    let mut output = vec!["".to_string(); n];

    for i in 0..vec_of_vectors.len() {
        output = output
            .iter()
            // Any shorter vector will re-cycle its values to the length of
            // longest one
            .zip(vec_of_vectors[i].iter().cycle())
            .map(|(x, y)| {
                // No need for a sep in the beginning
                if i == 0 {
                    return y.clone();
                }
                format!("{x}{sep}{y}")
            })
            .collect();
    }

    if collapse.len() > 0 {
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
        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<_> = vec!["1 2"];

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_02() {
        let mut env = Environment::default();
        let args = parse_args("paste(null)").unwrap();
        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<&str> = vec![];

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_03() {
        let mut env = Environment::default();
        let args = parse_args("paste(1.1, null, 2, false, 'a', c(1.0, 2.0), sep = '+')").unwrap();
        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<_> = vec!["1.1+2+false+a+1", "1.1+2+false+a+2"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_04() {
        let mut env = Environment::default();
        let args = parse_args("paste(1.1, null, 2, false, 'a', c(1.0, 2.0))").unwrap();
        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<_> = vec!["1.1 2 false a 1", "1.1 2 false a 2"]
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
        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
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
        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<_> = vec!["1.1 2 false a 1+1.1 2 false a 2"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_07() {
        let mut env = Environment::default();
        let args = parse_args("paste(1, 2, 3, collapse = '+')").unwrap();
        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<_> = vec!["1 2 3"].iter().map(|s| s.to_string()).collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_08() {
        let mut env = Environment::default();
        let args = parse_args("paste(c(1, 2), 3, 4, 5, sep = '-', collapse = '+')").unwrap();
        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<_> = vec!["1-3-4-5+2-3-4-5"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }
}
