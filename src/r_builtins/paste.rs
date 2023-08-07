use crate::ast::*;
use crate::lang::*;
use crate::r_vector::vectors::*;

pub fn primitive_paste(args: ExprList, env: &mut Environment) -> EvalResult {
    // TODO
    // Support lists

    // Need the sep and collapse parameters
    let sep_i = &args.keys.iter().position(|k| k == &Some("sep".to_string()));

    let R::List(mut vals) = env.eval_list(args)? else {
        unreachable!()
    };

    let sep_c = match sep_i {
        Some(i) => {
            let sep_val = vals.remove(*i);
            match sep_val {
                (_, R::Vector(Vector::Character(s_v))) => s_v.get(0).unwrap().clone(),
                _ => unreachable!(),
            }
        }
        // Default value for sep parameter is a space
        _ => OptionNA::Some(" ".to_string()),
    };

    let vals: Vec<_> = vals
        .into_iter()
        .map(|(k, v)| (k, v.force().unwrap_or(R::Null))) // TODO: raise this error
        .collect();

    // Coerce everything into strings
    let char_vals: Vec<R> = vals
        .iter()
        .map(|v| v.clone().1.as_character().unwrap())
        .collect();

    let vec_of_vectors: Vec<_> = char_vals
        .iter()
        .map(|v| v.get_vec_string())
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
                format!("{x}{sep_c}{y}")
            })
            .collect();
    }

    Ok(R::Vector(output.into()))
}

#[cfg(test)]
mod test_primitive_paste {
    use super::*;

    #[test]
    fn test_primitive_paste_01() {
        let mut env = Environment::default();

        // Making a value of args parameter of primitive_paste corresponding to R
        // c(NULL)
        let args = ExprList {
            keys: vec![None],
            values: vec![Expr::Null],
        };

        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<String> = vec![];

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_02() {
        let mut env = Environment::default();

        // Making a value of args parameter of primitive_paste corresponding to R
        // c(1.1, 2, FALSE, "a", c(1, 2), sep = "+")
        let args = ExprList {
            keys: vec![None, None, None, None, None, None, Some("sep".to_string())],
            values: vec![
                Expr::Number(1.1),
                Expr::Null,
                Expr::Integer(2),
                Expr::Bool(false),
                Expr::String("a".to_string()),
                Expr::Call(
                    Box::new(Expr::Symbol("c".to_string())),
                    ExprList {
                        keys: vec![None; 2],
                        values: vec![Expr::Number(1.0), Expr::Number(2.0)],
                    },
                ),
                // sep parameter
                Expr::String("+".to_string()),
            ],
        };

        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<_> = vec!["1.1+2+false+a+1", "1.1+2+false+a+2"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }

    #[test]
    fn test_primitive_paste_03() {
        let mut env = Environment::default();

        // Making a value of args parameter of primitive_paste corresponding to R
        // c(1.1, 2, FALSE, "a", c(1, 2))
        let args = ExprList {
            keys: vec![None; 6],
            values: vec![
                Expr::Number(1.1),
                Expr::Null,
                Expr::Integer(2),
                Expr::Bool(false),
                Expr::String("a".to_string()),
                Expr::Call(
                    Box::new(Expr::Symbol("c".to_string())),
                    ExprList {
                        keys: vec![None; 2],
                        values: vec![Expr::Number(1.0), Expr::Number(2.0)],
                    },
                ),
            ],
        };

        let observed = primitive_paste(args, &mut env).unwrap().get_vec_string();
        let expected: Vec<_> = vec!["1.1 2 false a 1", "1.1 2 false a 2"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(observed, expected);
    }
}
