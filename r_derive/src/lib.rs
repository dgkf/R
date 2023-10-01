extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, parse::Parse, LitStr, Expr};

#[derive(Clone)]
enum Builtin {
    Sym(Sym),
    Keyword,
}

#[derive(Clone)]
struct Sym {
    sym: LitStr,
    kind: Expr,
}

impl Parse for Builtin {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{parse_quote, ExprLit, Lit, MetaNameValue, Token};
        use syn::punctuated::Punctuated;

        let vars = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?.into_iter();
        let mut symbol: Option<LitStr> = None;
        let mut kind = parse_quote! { Function };

        for var in vars {
            match (var.path, var.value) {
                (k, Expr::Lit(ExprLit { lit: Lit::Str(s), .. })) if k.is_ident("sym") => {
                    symbol = Some(s);
                },
                (k, e) if k.is_ident("kind") => {
                    kind = e;
                }
                _ => todo!(),
            }
        }

        match symbol {
            Some(sym) => Ok(Builtin::Sym(Sym{ sym, kind })),
            None => Ok(Builtin::Keyword)
        }
    }
}

#[proc_macro_attribute]
pub fn builtin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as Builtin);
    let item = parse_macro_input!(item as DeriveInput);
    let what = item.ident.clone();

    let expanded = match attr {
        Builtin::Sym(Sym { sym, kind }) => quote! {
            #item

            #[automatically_derived]
            impl Sym for #what {
                const SYM: &'static str = #sym;
                const KIND: &'static SymKind = &SymKind::#kind;
            }

            #[automatically_derived]
            impl CallableClone for #what
            where
                Self: Callable
            {
                fn callable_clone(&self) -> Box<dyn Builtin> {
                    Box::new(self.clone())
                }
            }

            #[automatically_derived]
            impl Builtin for #what {}
        },
        Builtin::Keyword => quote! {
            #item

            #[automatically_derived]
            impl CallableClone for #what
            where
                Self: Callable
            {
                fn callable_clone(&self) -> Box<dyn Builtin> {
                    Box::new(self.clone())
                }
            }

            #[automatically_derived]
            impl Builtin for #what {}
        },
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Translate)]
pub fn derive_translate(_input: TokenStream) -> TokenStream {
    let output = quote! {
        impl Into<crate::parser::Rule> for Rule {
            fn into(self) -> crate::parser::Rule {
                match self {
                    Rule::loc_if => crate::parser::Rule::loc_if,
                    Rule::loc_else => crate::parser::Rule::loc_else,
                    Rule::loc_for => crate::parser::Rule::loc_for,
                    Rule::loc_while => crate::parser::Rule::loc_while,
                    Rule::loc_repeat => crate::parser::Rule::loc_repeat,
                    Rule::loc_return => crate::parser::Rule::loc_return,
                    Rule::loc_break => crate::parser::Rule::loc_break,
                    Rule::loc_continue => crate::parser::Rule::loc_continue,
                    Rule::loc_function => crate::parser::Rule::loc_function,
                    Rule::loc_fn => crate::parser::Rule::loc_fn,
                    Rule::loc_na => crate::parser::Rule::loc_na,
                    Rule::loc_null => crate::parser::Rule::loc_null,
                    Rule::loc_inf => crate::parser::Rule::loc_inf,
                    Rule::loc_true => crate::parser::Rule::loc_true,
                    Rule::loc_false => crate::parser::Rule::loc_false,
                    Rule::WS_NO_NL => crate::parser::Rule::WS_NO_NL,
                    Rule::WS => crate::parser::Rule::WS,
                    Rule::WB => crate::parser::Rule::WB,
                    Rule::CAPTURE_WS => crate::parser::Rule::CAPTURE_WS,
                    Rule::eoi => crate::parser::Rule::eoi,
                    Rule::hl => crate::parser::Rule::hl,
                    Rule::hl_kws => crate::parser::Rule::hl_kws,
                    Rule::hl_comment => crate::parser::Rule::hl_comment,
                    Rule::hl_control => crate::parser::Rule::hl_control,
                    Rule::hl_control_kws => crate::parser::Rule::hl_control_kws,
                    Rule::hl_reserved => crate::parser::Rule::hl_reserved,
                    Rule::hl_reserved_kws => crate::parser::Rule::hl_reserved_kws,
                    Rule::hl_value => crate::parser::Rule::hl_value,
                    Rule::hl_value_kws => crate::parser::Rule::hl_value_kws,
                    Rule::hl_call => crate::parser::Rule::hl_call,
                    Rule::hl_callname => crate::parser::Rule::hl_callname,
                    Rule::hl_sym => crate::parser::Rule::hl_sym,
                    Rule::hl_symbol_backticked => crate::parser::Rule::hl_symbol_backticked,
                    Rule::hl_str => crate::parser::Rule::hl_str,
                    Rule::hl_num => crate::parser::Rule::hl_num,
                    Rule::hl_infix => crate::parser::Rule::hl_infix,
                    Rule::hl_open => crate::parser::Rule::hl_open,
                    Rule::hl_brackets => crate::parser::Rule::hl_brackets,
                    Rule::hl_ops => crate::parser::Rule::hl_ops,
                    Rule::hl_other => crate::parser::Rule::hl_other,
                    Rule::repl => crate::parser::Rule::repl,
                    Rule::expr => crate::parser::Rule::expr,
                    Rule::comment => crate::parser::Rule::comment,
                    Rule::atomic => crate::parser::Rule::atomic,
                    Rule::prefixed => crate::parser::Rule::prefixed,
                    Rule::postfixed => crate::parser::Rule::postfixed,
                    Rule::infix => crate::parser::Rule::infix,
                    Rule::add => crate::parser::Rule::add,
                    Rule::subtract => crate::parser::Rule::subtract,
                    Rule::multiply => crate::parser::Rule::multiply,
                    Rule::divide => crate::parser::Rule::divide,
                    Rule::modulo => crate::parser::Rule::modulo,
                    Rule::power => crate::parser::Rule::power,
                    Rule::gt => crate::parser::Rule::gt,
                    Rule::gte => crate::parser::Rule::gte,
                    Rule::eq => crate::parser::Rule::eq,
                    Rule::neq => crate::parser::Rule::neq,
                    Rule::lt => crate::parser::Rule::lt,
                    Rule::lte => crate::parser::Rule::lte,
                    Rule::or => crate::parser::Rule::or,
                    Rule::vor => crate::parser::Rule::vor,
                    Rule::and => crate::parser::Rule::and,
                    Rule::vand => crate::parser::Rule::vand,
                    Rule::assign => crate::parser::Rule::assign,
                    Rule::special => crate::parser::Rule::special,
                    Rule::pipe => crate::parser::Rule::pipe,
                    Rule::dollar => crate::parser::Rule::dollar,
                    Rule::colon => crate::parser::Rule::colon,
                    Rule::doublecolon => crate::parser::Rule::doublecolon,
                    Rule::triplecolon => crate::parser::Rule::triplecolon,
                    Rule::prefix => crate::parser::Rule::prefix,
                    Rule::negate => crate::parser::Rule::negate,
                    Rule::postfix => crate::parser::Rule::postfix,
                    Rule::call => crate::parser::Rule::call,
                    Rule::index => crate::parser::Rule::index,
                    Rule::vector_index => crate::parser::Rule::vector_index,
                    Rule::block => crate::parser::Rule::block,
                    Rule::block_exprs => crate::parser::Rule::block_exprs,
                    Rule::block_sep => crate::parser::Rule::block_sep,
                    Rule::paren_expr => crate::parser::Rule::paren_expr,
                    Rule::atom => crate::parser::Rule::atom,
                    Rule::kw_function_or_fn => crate::parser::Rule::kw_function_or_fn,
                    Rule::kw_function => crate::parser::Rule::kw_function,
                    Rule::kw_if_else => crate::parser::Rule::kw_if_else,
                    Rule::kw_for => crate::parser::Rule::kw_for,
                    Rule::kw_while => crate::parser::Rule::kw_while,
                    Rule::kw_repeat => crate::parser::Rule::kw_repeat,
                    Rule::kw_break => crate::parser::Rule::kw_break,
                    Rule::kw_continue => crate::parser::Rule::kw_continue,
                    Rule::val_null => crate::parser::Rule::val_null,
                    Rule::val_na => crate::parser::Rule::val_na,
                    Rule::val_inf => crate::parser::Rule::val_inf,
                    Rule::val_true => crate::parser::Rule::val_true,
                    Rule::val_false => crate::parser::Rule::val_false,
                    Rule::number => crate::parser::Rule::number,
                    Rule::number_leading => crate::parser::Rule::number_leading,
                    Rule::number_trailing => crate::parser::Rule::number_trailing,
                    Rule::integer_expr => crate::parser::Rule::integer_expr,
                    Rule::integer => crate::parser::Rule::integer,
                    Rule::string_expr => crate::parser::Rule::string_expr,
                    Rule::single_quoted_string => crate::parser::Rule::single_quoted_string,
                    Rule::double_quoted_string => crate::parser::Rule::double_quoted_string,
                    Rule::single_quoted_string_char => crate::parser::Rule::single_quoted_string_char,
                    Rule::double_quoted_string_char => crate::parser::Rule::double_quoted_string_char,
                    Rule::escaped_char => crate::parser::Rule::escaped_char,
                    Rule::symbol => crate::parser::Rule::symbol,
                    Rule::symbol_with_backticks => crate::parser::Rule::symbol_with_backticks,
                    Rule::symbol_backticked => crate::parser::Rule::symbol_backticked,
                    Rule::symbol_ident => crate::parser::Rule::symbol_ident,
                    Rule::list => crate::parser::Rule::list,
                    Rule::pairs => crate::parser::Rule::pairs,
                    Rule::ellipsis => crate::parser::Rule::ellipsis,
                    Rule::elem => crate::parser::Rule::elem,
                    Rule::named => crate::parser::Rule::named,
                    Rule::vec => crate::parser::Rule::vec,
                }
            }
        }
    };

    output.into()
}
