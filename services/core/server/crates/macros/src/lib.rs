//! Procedural macros.
//! Check here <https://doc.rust-lang.org/reference/procedural-macros.html> for more information on procedural macros.

#![allow(clippy::needless_return)]

mod derive_env_variables;
mod derive_pg_enum;
mod derive_pg_text;
mod example_attr;
mod example_proc;

use proc_macro::TokenStream;

/// This derives a function [`load_from_env`] for a struct that loads the struct from environment variables
/// using the [`dotenvy`] crate.
///
/// ```rust, no_run
/// #[derive(Debug, Clone, EnvVariables)]
/// struct Config {
///     dev: bool,
///     database_url: String,
///     database_pool_size: u32,
/// }
///
/// dotenvy::dotenv().ok();
/// let config = Config::load_from_env();
/// ```
#[proc_macro_derive(EnvVariables)]
pub fn derive_env_variables(input: TokenStream) -> TokenStream {
    return derive_env_variables::derive_env_variables_impl(input);
}

/// This auto derives the diesel traits [`ToSql`] and [`FromSql`] for a enum type to be used with diesel.
/// This is a rust adapter for a custom PostgreSQL enum type.
///
/// ```sql, no_run
/// CREATE TYPE EXAMPLE_ENUM_TYPE AS ENUM ('A', 'B', 'C', 'D');
/// ```
///
/// ```rust, no_run
/// #[derive(AsExpression, Display, EnumString, FromSqlRow, PgEnum)]
/// #[diesel(sql_type = schema::sql_types::ExampleEnumType)]
/// enum ExampleEnumType {
///     A,
///     B,
///     C,
///     D,
/// }
/// ```
#[proc_macro_derive(PgEnum)]
pub fn derive_pg_enum(input: TokenStream) -> TokenStream {
    return derive_pg_enum::derive_pg_enum_impl(input);
}

/// This auto derives the diesel traits [`ToSql`] and [`FromSql`] for a enum type to be used with diesel.
/// This is a rust adapter for an arbitrary PostgreSQL text type.
///
/// ```rust, no_run
/// #[derive(AsExpression, Display, EnumString, FromSqlRow, PgText)]
/// #[diesel(sql_type = diesel::sql_types::Text)]
/// enum Thing {
///     A,
///     B,
///     C,
///     D,
/// }
/// ```
#[proc_macro_derive(PgText)]
pub fn derive_pg_text(input: TokenStream) -> TokenStream {
    return derive_pg_text::derive_pg_text_impl(input);
}

/// This is an example of a custom procedural macro.
///
/// ```rust, no_run
/// example_proc_macro!();
/// ```
#[proc_macro]
pub fn example_proc_macro(input: TokenStream) -> TokenStream {
    return example_proc::example_proc_macro_impl(input);
}

/// This is an example of a custom attribute macro.
///
/// ```rust, no_run
/// #[example_attr_macro(name = "foo", description = "bar")]
/// struct MyStruct;
///
/// assert_eq!(MyStruct::name(), "foo");
/// assert_eq!(MyStruct::description(), Some("bar"));
/// ```
#[proc_macro_attribute]
pub fn example_attr_macro(args: TokenStream, r#struct: TokenStream) -> TokenStream {
    return example_attr::example_attr_macro_impl(args, r#struct);
}
