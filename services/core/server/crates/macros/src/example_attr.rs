use proc_macro::TokenStream;
use quote::quote;

#[derive(Debug, darling::FromMeta)]
struct ExampleAttrArgs {
    name: String,
    description: Option<String>,
}

pub fn example_attr_macro_impl(args: TokenStream, r#struct: TokenStream) -> TokenStream {
    let args = match darling::ast::NestedMeta::parse_meta_list(args.into()) {
        Ok(x) => x,
        Err(e) => return e.into_compile_error().into(),
    };

    let args = match <ExampleAttrArgs as darling::FromMeta>::from_list(&args) {
        Ok(x) => x,
        Err(e) => return e.write_errors().into(),
    };

    let r#struct = syn::parse_macro_input!(r#struct as syn::ItemStruct);

    let struct_name = &r#struct.ident;
    let ExampleAttrArgs { name, description } = args;

    let expanded = quote! {
        #r#struct

        impl #struct_name {
            pub fn name() -> &'static str {
                #name
            }

            pub fn description() -> Option<&'static str> {
                #description
            }
        }
    };

    return TokenStream::from(expanded);
}
