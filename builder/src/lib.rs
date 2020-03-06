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
    let ident = ast.ident;

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

    let set_value_fields = fields.iter().map(|f| {
        if let Some(ident) = &f.ident {
            quote! {
                #ident: None
            }
        }
        else {
            unimplemented!()
        }
    });
    
    let option_fields = fields.iter().map(|f| {
        if let Some(ident) = &f.ident {
            let ty = &f.ty;
            quote!{
                #ident : std::option::Option<#ty>
            }
        }
        else {
            unimplemented!()
        }
    });
    // eprintln!("option fields: {:#?}", option_fields);

    let methods = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote!{
            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        }
    });
    
    let builder_ident = format_ident!("{}Builder", ident);
    let expanded = quote! {
        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #( #set_value_fields ,)*
                } 
            }
        }

        pub struct #builder_ident {
            #( #option_fields ,)*
        }

        impl #builder_ident {
            #( #methods )*
        }

    };
    TokenStream::from(expanded)
}
