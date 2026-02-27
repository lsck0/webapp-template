use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, parse_macro_input};

pub fn example_proc_macro_impl(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as Expr);

    let expanded = quote! {
        println!("This is an example of a custom procedural macro.");
    };

    return TokenStream::from(expanded);
}
