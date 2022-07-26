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

    let new_wildcard = if path.contains("/:") {
        let path_clone = path.clone();
        let mut iter = path_clone.split(":");
        path = iter.next().unwrap().to_string();
        let id = iter.next().unwrap().to_string();
        if path.len() != 1 {
            path.pop();
        };
        quote! {get_route = get_route.set_wildcard(#id.into());}
    } else {
        quote! {}
    };

    let new_get_body = if is_body_args {
        quote! {
            fn body(#body_args) -> Vec<u8> {
                #body.into()
            }

            get_route = get_route.set_body_with(body);
        }
    } else {
        quote! {
            fn body() -> Vec<u8> {
                #body.into()
            }

            get_route = get_route.set_body(body);
        }
    };

    let output = quote! {
        fn #name() -> Box<dyn Route> {
            let mut get_route = GetRoute::new()
                .set_path(#path.into())
                .set_is_args(#is_body_args);

            #new_wildcard
            #new_get_body

            Box::new(get_route)
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
    let new_wildcard = if path.contains("/:") {
        let path_clone = path.clone();
        let mut iter = path_clone.split(":");
        path = iter.next().unwrap().to_string();
        let id = iter.next().unwrap().to_string();
        if path.len() != 1 {
            path.pop();
        };
        quote! {post_route = post_route.set_wildcard(#id.into());}
    } else {
        quote! {}
    };

    let new_post_body = if is_body_args {
        quote! {
            fn body(#fn_args) -> Vec<u8> {
                #body.into()
            }

            post_route = post_route.set_body_with(body);
        }
    } else {
        quote! {
            fn body() -> Vec<u8> {
                #body.into()
            }

            post_route = post_route.set_body(body);
        }
    };
    let output = quote! {
        fn #name() -> Box<Route> {
            let mut post_route = PostRoute::new()
                .set_path(#path.into())
                .set_is_args(#is_body_args);

            #new_wildcard
            #new_post_body

            Box::new(post_route)
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
