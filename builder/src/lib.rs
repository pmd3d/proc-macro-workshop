use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields, Type, PathArguments, GenericArgument};

#[proc_macro_derive(Builder, attributes(builder))]
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
        
        if let Some(inner_ty) = extract_option_inner(ty) {
            quote! {
                #name: std::option::Option<#inner_ty>
            }
        } else {
            quote! {
                #name: std::option::Option<#ty>
            }
        }
    });
    
    fn mk_err<T: quote::ToTokens>(t: T) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
        Err(syn::Error::new_spanned(t, "expected `builder(each = \"...\")`").to_compile_error())
    }
    
    let builder_field_inits: Result<Vec<proc_macro2::TokenStream>, _> = fields.iter().map(|f| {
        let name = &f.ident;
    
        let each_attr = f.attrs.iter().find_map(|attr| {
            if attr.path.is_ident("builder") {
                match attr.parse_meta() {
                    Ok(syn::Meta::List(mut nvs)) => {
                        if nvs.nested.len() != 1 {
                            return Some(mk_err(nvs));
                        }
    
                        match nvs.nested.pop().unwrap().into_value() {
                            syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) => {
                                if nv.path.is_ident("each") {
                                    Some(Ok(quote! {
                                        #name: std::option::Option::Some(vec![])
                                    }))
                                } else {
                                    Some(mk_err(nvs))
                                }
                            },
                            meta => {
                                // nvs.nested[0] was not k = v
                                return Some(mk_err(meta));
                            }
                        }
                    }
                    Ok(meta) => {
                        // inside of #[] there was either just an identifier (`#[builder]`) or a key-value
                        // mapping (`#[builder = "foo"]`), neither of which are okay.
                        return Some(mk_err(meta));
                    }
                    Err(e) => {
                        return Some(Err(e.to_compile_error()));
                    }
                }
            } else {
                None
            }
        });
    
        match each_attr {
            Some(Ok(_)) => Ok(quote! {
                #name: std::option::Option::Some(vec![])
            }),
            Some(Err(error)) => Err(error),
            None => Ok(quote! {
                #name: std::option::Option::None
            }),
        }
    }).collect();
    
    let builder_field_inits = match builder_field_inits {
        Ok(inits) => inits,
        Err(e) => return e.into(),
    };
                    
    // Generate builder methods
    let builder_methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
    
        let each_name = f.attrs.iter()
            .filter_map(|attr| {
                if attr.path.is_ident("builder") {
                    match attr.parse_meta() {
                        Ok(syn::Meta::List(list)) => {
                            list.nested.iter().filter_map(|nested_meta| {
                                if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = nested_meta {
                                    if name_value.path.is_ident("each") {
                                        if let syn::Lit::Str(lit_str) = &name_value.lit {
                                            return Some(lit_str.value());
                                        }
                                    }
                                }
                                None
                            }).next()
                        },
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .next();
    
        if let Some(inner_ty) = extract_option_inner(ty) {
            if let Some(each_name) = each_name {
                let each_ident = syn::Ident::new(&each_name, proc_macro2::Span::call_site());
                quote! {
                    fn #each_ident(&mut self, #each_ident: #inner_ty) -> &mut Self {
                        if let std::option::Option::Some(v) = &mut self.#name {
                            v.push(#each_ident);
                        } else {
                            self.#name = std::option::Option::Some(vec![#each_ident]);
                        }
                        self
                    }
                }
            } else {
                quote! {
                    fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                        self.#name = std::option::Option::Some(#name);
                        self
                    }
                }
            }
        } else {
            if let Some(each_name) = each_name {
                let inner_ty = if let syn::Type::Path(type_path) = ty {
                    if let Some(segment) = type_path.path.segments.last() {
                        if let syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = angle_bracketed_generic_arguments.args.first() {
                                inner_type.clone()
                            } else {
                                ty.clone()
                            }
                        } else {
                            ty.clone()
                        }
                    } else {
                        ty.clone()
                    }
                } else {
                    ty.clone()
                };
                let each_ident = syn::Ident::new(&each_name, proc_macro2::Span::call_site());
                quote! {
                    fn #each_ident(&mut self, #each_ident: #inner_ty) -> &mut Self {
                        if let std::option::Option::Some(v) = &mut self.#name {
                            v.push(#each_ident);
                        } else {
                            self.#name = std::option::Option::Some(vec![#each_ident]);
                        }
                        self
                    }
                }
            } else {
                quote! {
                    fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = std::option::Option::Some(#name);
                        self
                    }
                }
            }
        }
    });
    
    // Generate build method
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        if is_option(ty) {
            quote! {
                let #name = self.#name.clone();
            }
        } else {
            quote! {
                let #name = self.#name.clone().ok_or_else(|| format!("Missing field: {}", stringify!(#name)))?;
            }
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
            msg: std::string::String,
        }

        impl std::fmt::Display for CommandBuilderError {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.msg)
            }
        }
        
        impl std::error::Error for CommandBuilderError {}
        
        impl From<std::string::String> for CommandBuilderError {
            fn from(s: std::string::String) -> Self {
                CommandBuilderError{ msg : s }
            }
        }

        impl CommandBuilder {
            #(#builder_methods)*

            pub fn build(&mut self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {
                #(#build_fields)*
                std::result::Result::Ok(#build_struct)
            }            
        }
    };

    // Return the generated code back to the compiler
    TokenStream::from(expanded)
}

// Helper function to check if the type is Option<T>
fn is_option(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        if let Some(segment) = tp.path.segments.last() {
            if segment.ident == "Option" {
                return true;
            }
        }
    }
    false
}

// Helper function to extract the inner type T from Option<T>
fn extract_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(tp) = ty {
        if let Some(segment) = tp.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if let Some(GenericArgument::Type(ref ty)) = args.args.first() {
                        return Some(ty);
                    }
                }
            }
        }
    }
    None
}










