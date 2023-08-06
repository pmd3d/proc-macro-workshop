use proc_macro::{TokenStream};
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields};
use quote::quote;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named_fields) => &named_fields.named,
            _ => panic!("Currently only support named fields"),
        },
        _ => panic!("Unsupported data type"),
    };

    let mut fields_debug = fields.iter().peekable();
    let mut debug_statements = quote! {};

    while let Some(field) = fields_debug.next() {
        let field_name = &field.ident;
        let debug_statement = quote! {
            write!(f, "{}: \"{}\"", stringify!(#field_name), &self.#field_name)?;
        };

        debug_statements.extend(debug_statement);

        if fields_debug.peek().is_some() {
            debug_statements.extend(quote! { write!(f, ", ")?; });
        }
    }

    let gen = quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {{ ", stringify!(#name))?;
                #debug_statements
                write!(f, " }}")
            }
        }
    };

    gen.into()
}
