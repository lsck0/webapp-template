use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn derive_env_variables_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = if let Data::Struct(data_struct) = input.data
        && let Fields::Named(fields) = data_struct.fields
    {
        fields.named
    } else {
        unimplemented!("Only structs with named fields are supported.")
    };

    let field_assignments = fields.into_iter().map(|field| {
        let field_name = field.ident.expect("every field to have an identifier");
        let env_name = field_name.to_string().to_uppercase();

        quote! {
            #field_name: dotenvy::var(#env_name)
                .expect(&format!("Environment variable {} not set", #env_name))
                .parse()
                .expect(&format!("Failed to parse environment variable {}", #env_name))
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn load_from_env() -> Self {
                Self {
                    #(#field_assignments),*
                }
            }
        }
    };

    return TokenStream::from(expanded);
}
