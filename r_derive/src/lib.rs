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
                (
                    k,
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }),
                ) if k.is_ident("sym") => {
                    symbol = Some(s);
                }
                (k, e) if k.is_ident("kind") => {
                    kind = e;
                }
                _ => todo!(),
            }
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
                fn is_transparent(&self) -> bool {
                    true
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
            }
        },
    };

    TokenStream::from(expanded)
}
