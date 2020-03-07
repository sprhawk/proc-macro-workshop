extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Data::Struct, DataStruct, DeriveInput,
    Field, Fields::Named, FieldsNamed, GenericArgument, PathArguments, PathSegment, Type, TypePath,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    // eprintln!("TOKENS: {}", input);
    let ast = parse_macro_input!(input as DeriveInput);
    // eprintln!("SYN: {:#?}", ast);
    // unimplemented!()
    let struct_ident = ast.ident;

    let fields = if let Struct(DataStruct {
        fields: Named(FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!();
    };

    // eprintln!("fields: {:#?}", fields);

    let struct_init_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: None
        }
    });

    // To handle optional fields
    let to_actual_type = |f: &Field| -> (syn::Type, bool) {
        // eprintln!("Field: {:#?}", f);
        if let Type::Path(TypePath {
            path: inner_path, ..
        }) = &f.ty
        {
            if inner_path.segments.len() == 1 {
                let seg = &inner_path.segments.first().unwrap();
                if seg.ident.to_string() == "Option" {
                    if let PathSegment {
                        arguments:
                            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                args: inner_args,
                                ..
                            }),
                        ..
                    } = seg
                    {
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

    // Generated:
    // #[derive(Builder)]
    // pub struct Command {
    //     executable: String,
    //     #[builder(each = "arg")]
    //     args: Vec<String>,
    //     #[builder(each = "env")]
    //     env: Vec<String>,
    //     current_dir: Option<String>,
    // }

    let builder_struct_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        let (actual_ty, _) = to_actual_type(&f);

        if f.attrs.len() > 0 {
            eprintln!("field {} attrs: {:#?}", ident.as_ref().unwrap().to_string(), f.attrs);
        }
        quote! {
            #ident : std::option::Option<#actual_ty>
        }
    });
    // eprintln!("option fields: {:#?}", option_fields);

    // Generated:
    // pub fn executable(&mut self, executable: String) -> &mut Self {
    //     self.executable = Some(executable);
    //     self
    // }
    // pub fn args(&mut self, args: Vec<String>) -> &mut Self {
    //     self.args = Some(args);
    //     self
    // }
    // pub fn env(&mut self, env: Vec<String>) -> &mut Self {
    //     self.env = Some(env);
    //     self
    // }
    // pub fn current_dir(&mut self, current_dir: String) -> &mut Self {
    //     self.current_dir = Some(current_dir);
    //     self
    // }

    let builder_methods = fields.iter().map(|f| {
        let ident = &f.ident;
        let (ty, _) = to_actual_type(&f);
        quote! {
            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        }
    });


    // Generated
    // pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
    //     Ok(Command {
    //         executable: self.executable.clone().expect("executable is not set"),
    //         args: self.args.clone().expect("args is not set"),
    //         env: self.env.clone().expect("env is not set"),
    //         current_dir: self.current_dir.clone(),
    //     })
    // }

    let builder_build_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        let (_, is_optional) = to_actual_type(&f);
        if is_optional {
            quote! {
                #ident: self.#ident.clone()
            }
        } else {
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
            #( #builder_struct_fields ,)*
        }

        impl #builder_ident {
            #( #builder_methods )*

            pub fn build(&mut self) -> Result<#struct_ident, Box<dyn std::error::Error>> {
                Ok(#struct_ident {
                    #( #builder_build_fields ,)*
                })
            }
        }

    };
    TokenStream::from(expanded)
}
