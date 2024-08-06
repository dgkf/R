extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, DeriveInput, Expr, LitStr};

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
        use syn::punctuated::Punctuated;
        use syn::{parse_quote, ExprLit, Lit, MetaNameValue, Token};

        let vars = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?.into_iter();
        let mut symbol: Option<LitStr> = None;
        let mut kind = parse_quote! { Function };

        for var in vars {
            match (var.path, var.value) {
                (k, Expr::Lit(ExprLit { lit: Lit::Str(s), .. })) => match k {
                    sym if sym.is_ident("sym") => symbol = Some(s),
                    _ => (),
                },
                (k, e) if k.is_ident("kind") => {
                    kind = e;
                }
                _ => todo!(),
            };
        }

        match symbol {
            Some(sym) => Ok(Builtin::Sym(Sym { sym, kind })),
            None => Ok(Builtin::Keyword),
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
            impl Builtin for #what {
                fn kind(&self) -> SymKind {
                    SymKind::#kind
                }
            }
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
            impl Builtin for #what {
                fn is_transparent(&self) -> bool {
                    true
                }

                fn kind(&self) -> SymKind {
                    SymKind::Keyword
                }
            }
        },
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Translate)]
pub fn derive_translate(_input: TokenStream) -> TokenStream {
    let output = quote! {
        extern crate self as __translate_r__;
        impl From<Rule> for __translate_r__::parser::en::Rule {
            fn from(rule: Rule) -> Self {
                use __translate_r__::parser::en;
                match rule {
                    Rule::loc_if => en::Rule::loc_if,
                    Rule::loc_else => en::Rule::loc_else,
                    Rule::loc_for => en::Rule::loc_for,
                    Rule::loc_in => en::Rule::loc_in,
                    Rule::loc_while => en::Rule::loc_while,
                    Rule::loc_repeat => en::Rule::loc_repeat,
                    Rule::loc_return => en::Rule::loc_return,
                    Rule::loc_break => en::Rule::loc_break,
                    Rule::loc_continue => en::Rule::loc_continue,
                    Rule::loc_function => en::Rule::loc_function,
                    Rule::loc_fn => en::Rule::loc_fn,
                    Rule::loc_na => en::Rule::loc_na,
                    Rule::loc_null => en::Rule::loc_null,
                    Rule::loc_inf => en::Rule::loc_inf,
                    Rule::loc_true => en::Rule::loc_true,
                    Rule::loc_false => en::Rule::loc_false,
                    Rule::WS_NO_NL => en::Rule::WS_NO_NL,
                    Rule::WS => en::Rule::WS,
                    Rule::WB => en::Rule::WB,
                    Rule::CAPTURE_WS => en::Rule::CAPTURE_WS,
                    Rule::eoi => en::Rule::eoi,
                    Rule::hl => en::Rule::hl,
                    Rule::hl_kws => en::Rule::hl_kws,
                    Rule::hl_comment => en::Rule::hl_comment,
                    Rule::hl_control => en::Rule::hl_control,
                    Rule::hl_control_kws => en::Rule::hl_control_kws,
                    Rule::hl_function => en::Rule::hl_function,
                    Rule::hl_function_kws => en::Rule::hl_function_kws,
                    Rule::hl_signal => en::Rule::hl_signal,
                    Rule::hl_signal_kws => en::Rule::hl_signal_kws,
                    Rule::hl_value => en::Rule::hl_value,
                    Rule::hl_value_kws => en::Rule::hl_value_kws,
                    Rule::hl_call => en::Rule::hl_call,
                    Rule::hl_callname => en::Rule::hl_callname,
                    Rule::hl_sym => en::Rule::hl_sym,
                    Rule::hl_symbol_backticked => en::Rule::hl_symbol_backticked,
                    Rule::hl_str => en::Rule::hl_str,
                    Rule::hl_num => en::Rule::hl_num,
                    Rule::hl_infix => en::Rule::hl_infix,
                    Rule::hl_open => en::Rule::hl_open,
                    Rule::hl_brackets => en::Rule::hl_brackets,
                    Rule::hl_ops => en::Rule::hl_ops,
                    Rule::hl_other => en::Rule::hl_other,
                    Rule::repl => en::Rule::repl,
                    Rule::expr => en::Rule::expr,
                    Rule::comment => en::Rule::comment,
                    Rule::atomic => en::Rule::atomic,
                    Rule::standalone => en::Rule::standalone,
                    Rule::prefixed => en::Rule::prefixed,
                    Rule::postfixed => en::Rule::postfixed,
                    Rule::infix => en::Rule::infix,
                    Rule::add => en::Rule::add,
                    Rule::subtract => en::Rule::subtract,
                    Rule::multiply => en::Rule::multiply,
                    Rule::divide => en::Rule::divide,
                    Rule::modulo => en::Rule::modulo,
                    Rule::power => en::Rule::power,
                    Rule::gt => en::Rule::gt,
                    Rule::gte => en::Rule::gte,
                    Rule::eq => en::Rule::eq,
                    Rule::neq => en::Rule::neq,
                    Rule::lt => en::Rule::lt,
                    Rule::lte => en::Rule::lte,
                    Rule::or => en::Rule::or,
                    Rule::vor => en::Rule::vor,
                    Rule::and => en::Rule::and,
                    Rule::vand => en::Rule::vand,
                    Rule::assign => en::Rule::assign,
                    Rule::special => en::Rule::special,
                    Rule::pipe => en::Rule::pipe,
                    Rule::dollar => en::Rule::dollar,
                    Rule::colon => en::Rule::colon,
                    Rule::doublecolon => en::Rule::doublecolon,
                    Rule::triplecolon => en::Rule::triplecolon,
                    Rule::prefix => en::Rule::prefix,
                    Rule::not => en::Rule::not,
                    Rule::postfix => en::Rule::postfix,
                    Rule::call => en::Rule::call,
                    Rule::index => en::Rule::index,
                    Rule::vector_index => en::Rule::vector_index,
                    Rule::block => en::Rule::block,
                    Rule::block_exprs => en::Rule::block_exprs,
                    Rule::block_sep => en::Rule::block_sep,
                    Rule::paren_expr => en::Rule::paren_expr,
                    Rule::atom => en::Rule::atom,
                    Rule::kw_function_or_fn => en::Rule::kw_function_or_fn,
                    Rule::kw_function => en::Rule::kw_function,
                    Rule::kw_if_else => en::Rule::kw_if_else,
                    Rule::kw_for => en::Rule::kw_for,
                    Rule::kw_while => en::Rule::kw_while,
                    Rule::kw_repeat => en::Rule::kw_repeat,
                    Rule::kw_return => en::Rule::kw_return,
                    Rule::kw_break => en::Rule::kw_break,
                    Rule::kw_continue => en::Rule::kw_continue,
                    Rule::val_null => en::Rule::val_null,
                    Rule::val_na => en::Rule::val_na,
                    Rule::val_inf => en::Rule::val_inf,
                    Rule::val_true => en::Rule::val_true,
                    Rule::val_false => en::Rule::val_false,
                    Rule::number => en::Rule::number,
                    Rule::number_leading => en::Rule::number_leading,
                    Rule::number_trailing => en::Rule::number_trailing,
                    Rule::more => en::Rule::more,
                    Rule::integer_expr => en::Rule::integer_expr,
                    Rule::integer => en::Rule::integer,
                    Rule::string_expr => en::Rule::string_expr,
                    Rule::single_quoted_string => en::Rule::single_quoted_string,
                    Rule::double_quoted_string => en::Rule::double_quoted_string,
                    Rule::single_quoted_string_char => en::Rule::single_quoted_string_char,
                    Rule::double_quoted_string_char => en::Rule::double_quoted_string_char,
                    Rule::escaped_char => en::Rule::escaped_char,
                    Rule::symbol => en::Rule::symbol,
                    Rule::symbol_with_backticks => en::Rule::symbol_with_backticks,
                    Rule::symbol_backticked => en::Rule::symbol_backticked,
                    Rule::symbol_ident => en::Rule::symbol_ident,
                    Rule::list => en::Rule::list,
                    Rule::pairs => en::Rule::pairs,
                    Rule::elem => en::Rule::elem,
                    Rule::named => en::Rule::named,
                    Rule::vec => en::Rule::vec,
                }
            }
        }
    };

    output.into()
}

#[proc_macro_derive(LocalizedParser)]
pub fn derive_localized_parser(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let what = item.ident.clone();

    let output = quote! {
        // Each parser needs its own PrattParser, implemented specifically
        // for its flavor of grammar rules.
        use pest::pratt_parser::PrattParser;
        use std::sync::OnceLock;
        fn pratt_parser() -> &'static PrattParser<Rule> {
            static PRATT: OnceLock<PrattParser<Rule>> = OnceLock::new();
            PRATT.get_or_init(|| {
                use pest::pratt_parser::{Assoc::*, Op};
                use Rule::*;

                // Precedence is defined lowest to highest
                pest::pratt_parser::PrattParser::new()
                    .op(Op::infix(assign, Right))
                    .op(Op::infix(or, Left) | Op::infix(vor, Left))
                    .op(Op::infix(and, Left) | Op::infix(vand, Left))
                    .op(Op::infix(lt, Left)
                        | Op::infix(gt, Left)
                        | Op::infix(lte, Left)
                        | Op::infix(gte, Left)
                        | Op::infix(eq, Left)
                        | Op::infix(neq, Left))
                    .op(Op::infix(add, Left) | Op::infix(subtract, Left))
                    .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
                    .op(Op::infix(modulo, Left) | Op::infix(special, Left) | Op::infix(pipe, Left))
                    .op(Op::infix(power, Left))
                    .op(Op::infix(colon, Left))
                    .op(Op::infix(dollar, Left))
            })
        }

        extern crate self as __localized_parser_r__;
        use __localized_parser_r__::error::Error;
        use __localized_parser_r__::lang:: Signal;
        use __localized_parser_r__::session:: SessionParserConfig;
        use __localized_parser_r__::parser::*;
        impl LocalizedParser for #what {
            fn parse_input_with(&self, input: &str, config: &SessionParserConfig) -> ParseResult {
                let pairs = <Self as pest::Parser<Rule>>::parse(Rule::repl, input);

                match pairs {
                    // comments currently entirely unparsed, return thunk
                    Ok(pairs) if pairs.len() == 0 => Err(Signal::Thunk),

                    // for any expressions
                    Ok(pairs) => parse_expr(config, self, pratt_parser(), pairs),
                    Err(e) => Err(Signal::Error(Error::from_parse_error(input, e))),
                }
            }

            fn parse_highlight_with(&self, input: &str, config: &SessionParserConfig) -> HighlightResult {
                let pairs = <Self as pest::Parser<Rule>>::parse(Rule::hl, input);
                match pairs {
                    Ok(pairs) => Ok(pairs
                        .map(|pair| {
                            (
                                pair.as_str().to_string(),
                                Into::<en::Rule>::into(pair.as_rule()).into(),
                            )
                        })
                        .collect()),
                    Err(e) => Err(Signal::Error(Error::from_parse_error(input, e))),
                }
            }
        }
    };

    output.into()
}
