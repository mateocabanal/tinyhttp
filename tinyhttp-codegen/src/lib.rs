use std::ops::Deref;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Ident, LitStr, Token,
};

struct RouteAttr {
    path: LitStr,
    cache: bool,
}

impl Parse for RouteAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path = input.parse::<LitStr>()?;
        let mut cache = false;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let ident = input.parse::<Ident>()?;

            if ident == "cache" {
                cache = true;
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "expected `cache`, e.g. #[get(\"/ping\", cache)]",
                ));
            }
        }

        Ok(RouteAttr { path, cache })
    }
}

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn: syn::ItemFn = syn::parse(item).unwrap();
    let attr: RouteAttr = syn::parse(attr).unwrap();
    let value = attr.path;

    let sig = item_fn.sig;
    let name = sig.ident.clone();
    let body = item_fn.block.deref();
    let return_type = sig.output;

    let fn_args = sig.inputs;
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

    if attr.cache && is_body_args {
        return syn::Error::new(
            name.span(),
            "`cache` can only be used on no-argument handlers",
        )
        .into_compile_error()
        .into();
    }

    let new_get_body = if attr.cache {
        quote! {
            let mut get_route = CachedRoute::new()
                .set_method(Method::GET)
                .set_path(#path.into());

            fn body() -> Response {
                #body.into()
            }

            get_route = get_route.set_body(body);
        }
    } else if is_body_args {
        let mut fn_args_iter = fn_args.iter();
        let first_arg_name = fn_args_iter.next().unwrap();
        let arg_type = match first_arg_name {
            syn::FnArg::Typed(i) => i.to_owned(),
            _ => todo!(),
        };

        quote! {
            let mut get_route = GetRouteWithReqAndRes::new()
                .set_path(#path.into());

            fn body<'b>(try_from_req: &'b mut Request, _sock: &'b mut std::net::TcpStream) -> Response {
                let #arg_type = try_from_req.into();
                #body.into()
            }

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
            #new_get_body
            #new_wildcard

            Box::new(get_route)
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: syn::ItemFn = syn::parse(item).unwrap();
    let attr: RouteAttr = syn::parse(attr).unwrap();
    let value = attr.path;

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

    if attr.cache && is_body_args {
        return syn::Error::new(
            name.span(),
            "`cache` can only be used on no-argument handlers",
        )
        .into_compile_error()
        .into();
    }

    let new_post_body = if attr.cache {
        quote! {
            let mut post_route = CachedRoute::new()
                .set_method(Method::POST)
                .set_path(#path.into());

            fn body() -> Response {
                #body.into()
            }

            post_route = post_route.set_body(body);
        }
    } else if is_body_args {
        let first_arg_name = fn_args.first().unwrap();
        let arg_type = match first_arg_name {
            syn::FnArg::Typed(i) => i.to_owned(),
            _ => todo!(),
        };

        quote! {
            let mut post_route = PostRouteWithReqAndRes::new()
                .set_path(#path.into());

            fn body<'b>(try_from_req: &'b mut Request, _sock: &'b mut std::net::TcpStream) -> Response {
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
            #new_post_body
            #new_wildcard

            Box::new(post_route)
        }
    };

    output.into()
}
