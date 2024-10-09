use crate::object::rep::Rep;
use crate::object::Obj;

pub type List = Rep<Obj>;

#[cfg(test)]
mod tests {
    use crate::{r, r_expect};
    #[test]
    fn list_declaration_ambiguity() {
        assert_eq!(r!((a = 1,)), r!(list(a = 1)));
        assert_eq!(r!((a = 1)), r!(1));
        assert_eq!(r!((1)), r!(1));
        assert_eq!(r!((1,)), r!(list(1)));
    }

    #[test]
    fn copy_on_write_single_bracket() {
        r_expect! {{"
            l1 = (1,)
            l2 = l1
            l1[1] = 2
            l1[[1]] == 2 & l2[[1]] == 1
        "}}
    }
    #[test]
    fn copy_on_write_double_bracket_names() {
        r_expect! {{r#"
            l1 = (a = 1,)
            l2 = l1
            l1[["a"]] = 2
            l1$a == 2 & l2$a == 1
        "#}}
    }
    #[test]
    fn copy_on_write_single_bracket_names() {
        r_expect! {{r#"
            l1 = (a = 1,)
            l2 = l1
            l1["a"] = 2
            l1$a == 2 & l2$a == 1
        "#}}
    }
    #[test]
    fn copy_on_write_slice_names() {
        r_expect! {{r#"
            l = (a = 1, b = 2, c = 3)
            l1 = l
            l1[c("a", "b")] = (10, 20)

            l1$a == 10 && l1$b == 20 & l$a == 1 & l$b == 2
        "#}}
    }
    #[test]
    fn copy_on_write_slice_indices() {
        r_expect! {{"
            l = (1, 2)
            l1 = l
            l1[1:2] = (10, 20)
            l1[[1]] == 10 && l1[[2]] == 20 & l[[1]] == 1 & l[[2]] == 2
        "}}
    }

    #[test]
    fn copy_on_write_index() {
        r_expect! {{"
            l = (1, 2)
            l_cow = l  # at this point, a copy-on-write reference
            l_cow[[2]] = 20
            l_cow[[1]] == 1 && l_cow[[2]] == 20 && l[[1]] == 1 && l[[2]] == 2
        "}}
    }

    #[test]
    fn nested_double_bracket_index() {
        r_expect! {{"
            l = ((1,),)
            l_cow = l
            l_cow[[1]][[1]] = 20
            l_cow[[1]][[1]] == 20 && l[[1]][[1]] == 1
        "}}
    }
    #[test]
    fn nested_double_bracket_names() {
        r_expect! {{r#"
            l = (a = (b = 1,),)
            l_cow = l
            l_cow[["a"]][["b"]] = 20
            l_cow[["a"]][["b"]] == 20 && l[["a"]][["b"]] == 1
        "#}}
    }
    #[test]
    fn nested_double_bracket_mixed() {
        r_expect! {{r#"
            l = (a = (1,),)
            l_cow = l
            l_cow[["a"]][[1]] = 20
            l_cow[["a"]][[1]] == 20 && l[["a"]][[1]] == 1
        "#}}
    }
    #[test]
    fn assign_list_to_list_slice() {
        r_expect! {{r#"
            l = (1, 2, 3)
            l[1:2] = (10, 20)
            l[[1]] == 10 & l[[2]] == 20
        "#}}
    }
    #[test]
    fn list_assign() {
        r_expect! {{r#"
            l = (function() null, )
            l[[1]] = 1
            l[[1]] == 1
        "#}}
    }
    #[test]
    fn assign_atomic_to_list_slice() {
        r_expect! {{r#"
            l = (1, 2, 3)
            l[1:2] = [3, 4]
            l[[1]] == 3 & l[[2]] == 4
        "#}}
    }
    #[test]
    fn assign_null() {
        r_expect! {{r#"
            l = (1, )
            l[[1]] = null
            is_null(l[[1]])
        "#}}
    }
    #[test]
    #[should_panic]
    fn index_length_one() {
        r_expect! {{r#"
            list(1, 2)[[c(1L, 2L)]]
        "#}}
    }
}
