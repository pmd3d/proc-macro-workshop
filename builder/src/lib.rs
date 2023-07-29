use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

     // Used to get the identifier of the struct (e.g., "Command")
     let name = &input.ident;

     // Get the fields of the struct
     let fields = if let 
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) = &input.data
    {
        &fields.named
    } else {
        // You can add more code here to provide a better error message
        panic!("Currently only supports structs with named fields");
    };

    // Generate builder field types
    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: Option<#ty>
        }
    });

    // Generate builder field initialization to None
    let builder_field_inits = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None
        }
    });

    // Generate builder methods
    let builder_methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    // Generate build method
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            let #name = self.#name.clone().ok_or_else(|| format!("Missing field: {}", stringify!(#name)))?;
        }
    });

    let builder_method_field_inits = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name : #name
        }
    });
    
    let build_struct = quote! {
        #name {
            #(#builder_method_field_inits),*
        }
    };

    // Generate the code to provide
    let expanded = quote! {

        pub struct CommandBuilder {
            #(#builder_fields),*
        }

        impl #name {
            pub fn builder() -> CommandBuilder {
                CommandBuilder {
                    #(#builder_field_inits),*
                }
            }
        }

        #[derive(Debug)]
        pub struct CommandBuilderError {
            msg: String,
        }

        impl std::fmt::Display for CommandBuilderError {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.msg)
            }
        }
        
        impl std::error::Error for CommandBuilderError {}
        
        impl From<String> for CommandBuilderError {
            fn from(s: String) -> Self {
                CommandBuilderError{ msg : s }
            }
        }

        impl CommandBuilder {
            #(#builder_methods)*

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                #(#build_fields)*
                Ok(#build_struct)
            }            
        }
    };

    // Return the generated code back to the compiler
    TokenStream::from(expanded)
}
