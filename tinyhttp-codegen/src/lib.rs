use std::ops::Deref;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    //eprintln!("{:#?}\n{:#?}", attr, item);
    let args = parse_macro_input!(attr as syn::AttributeArgs);
    let item: syn::ItemFn = syn::parse(item).unwrap();
    let sig = item.sig;
    let name = sig.ident.clone();
    let body = item.block.deref();
    let path_token = args[0].clone();

    let body_args = sig.inputs;
    let is_body_args = !body_args.is_empty();

    //eprintln!("LEN: {}", body_args.len());

    let mut path;
    match path_token.clone() {
        syn::NestedMeta::Meta(_) => todo!(),
        syn::NestedMeta::Lit(e) => match e {
            syn::Lit::Str(e) => {
                path = e.value();
            }
            syn::Lit::ByteStr(_) => todo!(),
            syn::Lit::Byte(_) => todo!(),
            syn::Lit::Char(_) => todo!(),
            syn::Lit::Int(_) => todo!(),
            syn::Lit::Float(_) => todo!(),
            syn::Lit::Bool(_) => todo!(),
            syn::Lit::Verbatim(_) => todo!(),
        },
    };
    let wildcard = if path.contains(":") {
        let path2 = path.clone();
        let mut iter = path2.split(":");
        path = iter.next().unwrap().to_string();
        let id = iter.next().unwrap().to_string();
        quote! {Some(#id.into())}
    } else {
        quote! {None}
    };

    let get_body = if is_body_args {
        quote! {
        fn get_body(&self) -> Option<fn() -> Vec<u8>> {
            None
        }
        fn get_body_with(&self) -> Option<fn(Request) -> Vec<u8>> {
            fn body(#body_args) -> Vec<u8> {
                #body.into()
            }
            Some(body)
            }
        }
    } else {
        quote! {
        fn get_body_with(&self) -> Option<fn(Request) -> Vec<u8>> {
            None
        }
        fn get_body(&self) -> Option<fn() -> Vec<u8>> {
            fn body() -> Vec<u8> {
                #body.into()
            }
            Some(body)
            }
        }
    };

    let output = quote! {
        fn #name() -> Box<Route> {

            #[allow(non_camel_case_types)]

            #[derive(Clone, Debug)]
            struct route {
                path: &'static str,
                method: Method,
                wildcard: Option<String>,
                is_args: bool,
            }

            impl route {
                fn new() -> Self {
                    route {
                        path: #path.into(),
                        method: Method::GET,
                        wildcard: #wildcard,
                        is_args: #is_body_args,
                    }
                }
            }

            impl Default for route {
                fn default() -> route {
                    route {
                        path: #path.into(),
                        method: Method::GET,
                        wildcard: #wildcard,
                        is_args: #is_body_args,
                    }
                }
            }

            impl Route for route {
                fn get_path(&self) -> &str {
                    self.path
                }
                fn get_method(&self) -> Method {
                    self.method
                }
                #get_body
                fn post_body(&self) -> Option<fn() -> Vec<u8>> {
                    None
                }
                fn post_body_with(&self) -> Option<fn(Request) -> Vec<u8>> {
                    None
                }
                fn wildcard(&self) -> Option<String> {
                    self.wildcard.clone()
                }
                fn is_args(&self) -> bool {
                    self.is_args
                }
                fn clone_dyn(&self) -> Box<dyn Route> {
                    Box::new(self.clone())
                }
            }

            Box::new(route::new())
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    //eprintln!("{:#?}\n{:#?}", attr, item);
    let args = parse_macro_input!(attr as syn::AttributeArgs);
    let item: syn::ItemFn = syn::parse(item).unwrap();

    let fn_args = item.sig.inputs;
    let name = item.sig.ident.clone();
    let body = item.block.deref();

    let path_token = args[0].clone();
    let is_body_args = !fn_args.is_empty();

    let mut path;
    match path_token.clone() {
        syn::NestedMeta::Meta(_) => todo!(),
        syn::NestedMeta::Lit(e) => match e {
            syn::Lit::Str(e) => {
                path = e.value();
            }
            syn::Lit::ByteStr(_) => todo!(),
            syn::Lit::Byte(_) => todo!(),
            syn::Lit::Char(_) => todo!(),
            syn::Lit::Int(_) => todo!(),
            syn::Lit::Float(_) => todo!(),
            syn::Lit::Bool(_) => todo!(),
            syn::Lit::Verbatim(_) => todo!(),
        },
    };
    let wildcard = if path.contains(":") {
        let path2 = path.clone();
        let mut iter = path2.split(":");
        path = iter.next().unwrap().to_string();
        let id = iter.next().unwrap().to_string();
        quote! {Some(#id.into())}
    } else {
        quote! {None}
    };
    let post_body = if is_body_args {
        quote! {
        fn post_body(&self) -> Option<fn() -> Vec<u8>> {
            None
        }
        fn post_body_with(&self) -> Option<fn(Request) -> Vec<u8>> {
            fn body(#fn_args) -> Vec<u8> {
                #body.into()
            }
            Some(body)
            }
        }
    } else {
        quote! {
        fn post_body_with(&self) -> Option<fn(Request) -> Vec<u8>> {
            None
        }
        fn post_body(&self) -> Option<fn() -> Vec<u8>> {
            fn body() -> Vec<u8> {
                #body.into()
            }
            Some(body)
            }
        }
    };
    let output = quote! {
        fn #name() -> Box<Route> {

            #[allow(non_camel_case_types)]

            #[derive(Clone, Debug)]
            struct route {
                path: &'static str,
                method: Method,
                wildcard: Option<String>,
                is_args: bool,
            }

            impl route {
                fn new() -> Self {
                    route {
                        path: #path.into(),
                        method: Method::POST,
                        wildcard: #wildcard,
                        is_args: #is_body_args,
                    }
                }
            }

            impl Default for route {
                fn default() -> route {
                    route {
                        path: #path.into(),
                        method: Method::POST,
                        wildcard: #wildcard,
                        is_args: #is_body_args,
                    }
                }
            }

            impl Route for route {
                fn get_path(&self) -> &str {
                    self.path
                }
                fn get_method(&self) -> Method {
                    self.method
                }
                fn get_body(&self) -> Option<fn() -> Vec<u8>> {
                    None
                }
                fn get_body_with(&self) -> Option<fn(Request) -> Vec<u8>> {
                    None
                }

                /*fn post_body(&self) -> fn(Vec<u8>) -> Vec<u8> {
                    #body.into()
                }*/

                #post_body
                fn wildcard(&self) -> Option<String> {
                    self.wildcard.clone()
                }
                fn is_args(&self) -> bool {
                    self.is_args
                }
                fn clone_dyn(&self) -> Box<dyn Route> {
                    Box::new(self.clone())
                }
            }

            Box::new(route::new())
        }
    };

    /*let output = quote! {
        fn #name() -> (String, Vec<u8>, Method) {
            fn body() #output {
                #body
            }

            (#path.into(), body().into(), Method::POST)
        }
    };*/

    output.into()
}
