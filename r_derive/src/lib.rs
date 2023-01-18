extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

#[proc_macro_derive(Primitive)]
pub fn derive_primitive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let what = input.ident;

    let expanded = quote! {
        #[automatically_derived]
        impl CallableClone for #what
        where
            Self: Callable
        {
            fn callable_clone(&self) -> Box<dyn Primitive> {
                Box::new(self.clone())
            }
        }

        #[automatically_derived]
        impl Primitive for #what {}
    };

    TokenStream::from(expanded)
}
