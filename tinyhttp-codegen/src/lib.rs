use std::ops::Deref;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn: syn::ItemFn = syn::parse(item).unwrap();
    let value: syn::LitStr = syn::parse(attr).unwrap();

    let sig = item_fn.sig;
    let name = sig.ident.clone();
    let body = item_fn.block.deref();
    let return_type = sig.output;

    let fn_args = sig.inputs;
    let is_body_args = !fn_args.is_empty();
    //eprintln!("LEN: {}", body_args.len());

    let mut path = value.value();
    /*match path_token {
        syn::NestedMeta::Meta(_) => panic!("IN TOKEN MATCH!"),
        syn::NestedMeta::Lit(e) => match e {
            syn::Lit::Str(e) => {
                path = e.value();
            }
            syn::Lit::ByteStr(_) => panic!("IN TOKEN MATCH!"),
            syn::Lit::Byte(_) => panic!("IN TOKEN MATCH!"),
            syn::Lit::Char(_) => panic!("IN TOKEN MATCH!"),
            syn::Lit::Int(_) => panic!("IN TOKEN MATCH!"),
            syn::Lit::Float(_) => panic!("IN TOKEN MATCH!"),
            syn::Lit::Bool(_) => panic!("IN TOKEN MATCH!"),
            syn::Lit::Verbatim(_) => panic!("IN TOKEN MATCH!"),
        },
    };*/

    let new_wildcard = if path.contains("/:") {
        let path_clone = path.clone();
        let mut iter = path_clone.split(':');
        path = iter.next().unwrap().to_string();
        let id = iter.next().unwrap().to_string();
        if path.len() != 1 {
            path.pop();
        };
        quote! {get_route = get_route.set_wildcard(#id.into());}
    } else {
        quote! {}
    };

    let span = return_type.span();
    let return_error = match return_type {
        syn::ReturnType::Default => Some(
            syn::Error::new(span, "You're forgetting to return something...").into_compile_error(),
        ),
        _ => None,
    };

    if let Some(e) = return_error {
        return e.into();
    }

    // let is_ret_type_res = return_type_str == "Response";

    //    let new_get_body = if is_ret_type_res {
    //        quote! {
    //            let mut get_route = GetRouteWithReqAndRes::new()
    //                .set_path(#path.into());
    //
    //            fn body(#body_args) -> Response {
    //                #body.into()
    //            }
    //
    //            get_route = get_route.set_body(body);
    //        }
    let new_get_body = if is_body_args {
        let first_arg_name = fn_args.first().unwrap();
        let arg_type = match first_arg_name {
            syn::FnArg::Typed(i) => i.to_owned(),
            _ => todo!(),
        };
        quote! {
            let mut get_route = GetRouteWithReqAndRes::new()
                .set_path(#path.into());

            fn body<'b>(try_from_req: &'b mut Request) -> Response {
                let #arg_type = try_from_req.into();
                #body.into()
            }

            // OG
            // fn body(#body_args) -> Response {
            // #body.into()
            // }

            get_route = get_route.set_body(body);
        }
    } else {
        quote! {
            let mut get_route = BasicGetRoute::new()
                .set_path(#path.into());

            fn body() -> Response {
                #body.into()
            }

            get_route = get_route.set_body(body);
        }
    };

    let output = quote! {
        fn #name() -> Box<dyn Route> {
            /*let mut get_route = GetRoute::new()
                .set_path(#path.into())
                .set_is_args(#is_body_args)
                .set_is_ret_res(#is_ret_type_res);*/


            #new_get_body
            #new_wildcard

            Box::new(get_route)
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    //eprintln!("{:#?}\n{:#?}", attr, item);
    let item: syn::ItemFn = syn::parse(item).unwrap();
    let value: syn::LitStr = syn::parse(attr).unwrap();

    let fn_args = item.sig.inputs;
    let name = item.sig.ident.clone();
    let body = item.block.deref();
    let return_type = item.sig.output;

    let is_body_args = !fn_args.is_empty();

    let mut path = value.value();
    let new_wildcard = if path.contains("/:") {
        let path_clone = path.clone();
        let mut iter = path_clone.split(':');
        path = iter.next().unwrap().to_string();
        let id = iter.next().unwrap().to_string();
        if path.len() != 1 {
            path.pop();
        };
        quote! {post_route = post_route.set_wildcard(#id.into());}
    } else {
        quote! {}
    };

    let return_error = match return_type {
        syn::ReturnType::Default => Some(
            syn::Error::new(
                return_type.span(),
                "You're forgetting to return something...",
            )
            .into_compile_error(),
        ),
        _ => None,
    };

    if let Some(e) = return_error {
        return e.into();
    }

    let new_post_body = if is_body_args {
        let first_arg_name = fn_args.first().unwrap();
        let arg_type = match first_arg_name {
            syn::FnArg::Typed(i) => i.to_owned(),
            _ => todo!(),
        };
        // NOTE: Gets arg name and type
        //        let arg_name_pat = match &arg_type.pat.deref() {
        //            syn::Pat::Ident(i) => i.to_owned(),
        //            _ => todo!(),
        //        };
        //        let arg_name_type = match &arg_type.ty.deref() {
        //            syn::Type::Path(i) => i.to_owned(),
        //            _ => todo!(),
        //        };
        //
        //        let arg_name_type = &arg_name_type.path.segments.first().unwrap().ident;
        quote! {
            let mut post_route = PostRouteWithReqAndRes::new()
                .set_path(#path.into());

            fn body<'b>(try_from_req: &'b mut Request) -> Response {
                let #arg_type = try_from_req.into();
                #body.into()
            }

            post_route = post_route.set_body(body);
        }
    } else {
        quote! {
            let mut post_route = BasicPostRoute::new()
                .set_path(#path.into());

            fn body() -> Response {
                #body.into()
            }

            post_route = post_route.set_body(body);
        }
    };

    let output = quote! {
        fn #name() -> Box<dyn Route> {
            /*let mut post_route = PostRoute::new()
                .set_path(#path.into())
                .set_is_args(#is_body_args)
                .set_is_ret_res(#is_ret_type_res);*/

            #new_post_body
            #new_wildcard

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
