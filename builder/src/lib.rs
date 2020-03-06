extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, DataStruct, Data::{Struct}, Fields::{Named}, FieldsNamed};
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
    
    let builder_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote!{
            #ident : std::option::Option<#ty>
        }
    });
    // eprintln!("option fields: {:#?}", option_fields);

    let builder_methods = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote!{
            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        // let ty = &f.ty;
        quote! {
            #ident: self.#ident.clone().expect(concat!(stringify!(#ident), " is not set"))
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
