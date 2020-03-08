extern crate proc_macro;
use proc_macro2::{Ident, Span};
use self::proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Data::Struct, DataStruct, DeriveInput,
    Field, Fields::Named, FieldsNamed, GenericArgument, PathArguments, PathSegment, Type, TypePath,
    LitStr, Token, parse::{ Parse, ParseStream, Result }, 
};

#[derive(Debug)]
struct BuilderAttribute {
    ident: Ident,
    name: LitStr,
}

impl Parse for BuilderAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let name: LitStr = input.parse()?;
        if name.value() != "each" {
            syn::Error::new(input.span(), "expected `builder(each = \"...\")`").to_compile_error();
        }
        Ok(BuilderAttribute {
            ident,
            name,
        })
    }
}

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
        if let Type::Path(TypePath{ path ,.. } ) = &f.ty {
            // eprintln!("Path: {:#?}", path);
            if path.segments.len() == 1 {
                let p =  path.segments.first().unwrap();
                if p.ident == "Vec" {
                    return quote! { #ident: std::option::Option::<Vec<String>>::Some(Vec::<String>::new()) };
                }
            }
        }
        quote! {
            #ident: std::option::Option::<String>::None
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
        let field_ident = &f.ident;
        let (ty, _) = to_actual_type(&f);

        let mut tokenstream = 
            quote! {
                pub fn #field_ident(&mut self, #field_ident: #ty) -> &mut Self {
                    self.#field_ident = Some(#field_ident);
                    self
                }
            };
        
        for attr in f.attrs.iter() {
            let segs = &attr.path.segments;
            if segs.len() == 1 {
                let seg = segs.first().unwrap();
                let attr_ident = &seg.ident;
                if attr_ident == "builder"  {
                    // let meta = attr.tokens.parse_meta().expect("only attribute name 'each' is supported");
                    let parsed = attr.parse_args::<BuilderAttribute>().expect("not parsed");
                    // eprintln!("parsed meta: {:#?}", parsed);

                    if parsed.ident == "each" {
                        let name = parsed.name.value();
                        let arg_ident = Ident::new(&name, Span::call_site());
                        let ts = quote! {
                            fn #arg_ident(&mut self, #arg_ident: String) -> &mut Self {
                                if self.#field_ident.is_none() {
                                    /// this is hard coded type
                                    self.#field_ident = Some(Vec::<String>::new());
                                }
                                if let Some(a) = &mut self.#field_ident {
                                    a.push(#arg_ident);
                                }
                                self
                            }
                        };
                        if let Some(ident) = field_ident {
                            let field_name = ident.to_string();
                            if name == field_name {
                                tokenstream = ts;
                            }
                            else {
                                tokenstream.extend(ts);
                            }
                        }
                        // eprintln!("arg name: {:#?}", name);
                    }
                }
            }
            // eprintln!("field {} attr: {:#?}", ident.as_ref().unwrap().to_string(), attr);
            
        }

        tokenstream        
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

            pub fn build(&mut self) -> std::result::Result<#struct_ident, std::boxed::Box<dyn std::error::Error>> {
                std::result::Result::<#struct_ident, std::boxed::Box<dyn std::error::Error>>::Ok(#struct_ident {
                    #( #builder_build_fields ,)*
                })
            }
        }

    };
    TokenStream::from(expanded)
}
