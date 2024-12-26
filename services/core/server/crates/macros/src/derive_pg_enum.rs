use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_pg_enum_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl diesel::serialize::ToSql<crate::schema::sql_types::#name, diesel::pg::Pg> for #name {
            fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
                use std::io::Write;
                out.write_all(self.to_string().as_bytes())?;
                return Ok(diesel::serialize::IsNull::No);
            }
        }

        impl diesel::deserialize::FromSql<crate::schema::sql_types::#name, diesel::pg::Pg> for #name {
            fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
                use std::str::FromStr;
                let s = String::from_utf8(bytes.as_bytes().to_vec())?;
                return Self::from_str(&s).map_err(Into::into);
            }
        }
    };

    return TokenStream::from(expanded);
}
