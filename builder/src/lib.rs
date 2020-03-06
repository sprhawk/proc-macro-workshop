extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, DataStruct, Data::{Struct}, Fields::{Named}, FieldsNamed, Field, Type, TypePath, PathArguments, GenericArgument, AngleBracketedGenericArguments, PathSegment};
use quote::{quote, format_ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // eprintln!("TOKENS: {}", input);
    let ast = parse_macro_input!(input as DeriveInput);
    // eprintln!("SYN: {:#?}", ast);
    // unimplemented!()
    let struct_ident = ast.ident;

    let fields = if let Struct(
        DataStruct{
            fields: Named (
                    FieldsNamed {
                        ref named, ..
                    }
            ), ..
        }
    ) = ast.data
    {
        named
    }
    else {
        unimplemented!();
    };
    
    // eprintln!("fields: {:#?}", fields);

    let struct_init_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: None
        }
    });

    let to_actual_type = |f: &Field| -> (syn::Type, bool) {
        // eprintln!("Field: {:#?}", f);
        if let Type::Path(TypePath {
            path: inner_path, ..}) = &f.ty {
            if inner_path.segments.len() == 1 {
                let seg = &inner_path.segments.first().unwrap();
                if seg.ident.to_string() == "Option" {
                    if let PathSegment {
                        arguments: PathArguments::AngleBracketed(
                            AngleBracketedGenericArguments {
                                args: inner_args, ..
                            }), ..
                    } = seg {
                        if inner_args.len() == 1 {
                            if let GenericArgument::Type(ty) = &inner_args.first().unwrap() {
                                return (ty.clone(), true);
                            }
                        }
                    }
                }
            }
        }
        return (f.ty.clone(), false);
    };
    
    let builder_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        let (actual_ty, _) = to_actual_type(&f);
        
        quote!{
            #ident : std::option::Option<#actual_ty>
        }
    });
    // eprintln!("option fields: {:#?}", option_fields);

    let builder_methods = fields.iter().map(|f| {
        let ident = &f.ident;
        let (ty, _) = to_actual_type(&f);
        quote!{
            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        let (_, is_optional) = to_actual_type(&f);
        if is_optional {
            quote! {
                #ident: self.#ident.clone()
            }
        }
        else {
            quote! {
                #ident: self.#ident.clone().expect(concat!(stringify!(#ident), " is not set"))
            }
        }
    });

    let builder_ident = format_ident!("{}Builder", struct_ident);
    let expanded = quote! {
        impl #struct_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #( #struct_init_fields ,)*
                } 
            }
        }

        pub struct #builder_ident {
            #( #builder_fields ,)*
        }

        impl #builder_ident {
            #( #builder_methods )*

            pub fn build(&mut self) -> Result<#struct_ident, Box<dyn std::error::Error>> {
                Ok(#struct_ident {
                    #( #build_fields ,)*
                })
            }
        }

    };
    TokenStream::from(expanded)
}
