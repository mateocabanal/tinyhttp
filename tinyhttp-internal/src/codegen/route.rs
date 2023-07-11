use crate::config::{Method, Route, ToResponse};
use crate::request::Request;
use crate::response::Response;

#[cfg(test)]
use std::any::Any;

#[derive(Clone, Debug)]
pub struct BasicGetRoute {
    path: Option<&'static str>,
    method: Method,
    wildcard: Option<String>,
    is_args: Option<bool>,
    get_body: Option<fn() -> Vec<u8>>,
    get_body_with: Option<fn(Request) -> Vec<u8>>,

    get_body_with_res: Option<fn(Request) -> Response>,
    is_ret_res: bool,
}

impl Default for BasicGetRoute {
    fn default() -> Self {
        BasicGetRoute {
            path: None,
            method: Method::GET,
            wildcard: None,
            is_args: None,
            get_body: None,
            get_body_with: None,
            get_body_with_res: None,
            is_ret_res: false,
        }
    }
}

impl ToResponse for BasicGetRoute {
    fn to_res(&self, _res: Request) -> Response {
        Response::new()
            .body(self.get_body.unwrap()())
            .status_line("HTTP/1.1 200 OK\r\n")
            .mime("text/plain")
    }
}

impl BasicGetRoute {
    pub fn new() -> BasicGetRoute {
        Default::default()
    }
    pub fn set_path(mut self, path: &'static str) -> Self {
        self.path = Some(path);
        self
    }
    pub fn set_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
    pub fn set_wildcard(mut self, wildcard: String) -> Self {
        self.wildcard = Some(wildcard);
        self
    }
    pub fn set_is_args(mut self, is_args: bool) -> Self {
        self.is_args = Some(is_args);
        self
    }
    pub fn set_body(mut self, body: fn() -> Vec<u8>) -> Self {
        self.get_body = Some(body);
        self
    }
    pub fn set_body_with(mut self, body: fn(Request) -> Vec<u8>) -> Self {
        self.get_body_with = Some(body);
        self
    }
    pub fn set_body_with_res(mut self, body: fn(Request) -> Response) -> Self {
        self.get_body_with_res = Some(body);
        self
    }
    pub fn set_is_ret_res(mut self, is_ret_res: bool) -> Self {
        self.is_ret_res = is_ret_res;
        self
    }
}

impl Route for BasicGetRoute {
    fn get_path(&self) -> &str {
        self.path.unwrap()
    }
    fn get_method(&self) -> Method {
        self.method
    }
    fn wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
    fn clone_dyn(&self) -> Box<dyn Route> {
        Box::new(self.clone())
    }

    #[cfg(test)]
    fn any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct GetRouteWithReq {
    path: Option<&'static str>,
    method: Method,
    wildcard: Option<String>,
    get_body: Option<fn(Request) -> Vec<u8>>,
}

impl Default for GetRouteWithReq {
    fn default() -> Self {
        GetRouteWithReq {
            path: None,
            method: Method::GET,
            wildcard: None,
            get_body: None,
        }
    }
}

impl ToResponse for GetRouteWithReq {
    fn to_res(&self, res: Request) -> Response {
        Response::new()
            .body(self.get_body().unwrap()(res))
            .status_line("HTTP/1.1 200 OK\r\n")
            .mime("text/plain")
    }
}

impl GetRouteWithReq {
    pub fn new() -> GetRouteWithReq {
        Default::default()
    }
    pub fn set_path(mut self, path: &'static str) -> Self {
        self.path = Some(path);
        self
    }
    pub fn set_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
    pub fn set_wildcard(mut self, wildcard: String) -> Self {
        self.wildcard = Some(wildcard);
        self
    }
    pub fn set_body(mut self, body: fn(Request) -> Vec<u8>) -> Self {
        self.get_body = Some(body);
        self
    }

    pub fn get_body(&self) -> Option<fn(Request) -> Vec<u8>> {
        self.get_body
    }
}

impl Route for GetRouteWithReq {
    fn clone_dyn(&self) -> Box<dyn Route> {
        Box::new(self.clone())
    }
    fn get_method(&self) -> Method {
        self.method
    }
    fn get_path(&self) -> &str {
        self.path.unwrap()
    }
    fn wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
    #[cfg(test)]
    fn any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct GetRouteWithReqAndRes {
    path: Option<&'static str>,
    method: Method,
    wildcard: Option<String>,
    get_body: Option<fn(Request) -> Response>,
}

impl Default for GetRouteWithReqAndRes {
    fn default() -> Self {
        GetRouteWithReqAndRes {
            path: None,
            method: Method::GET,
            wildcard: None,
            get_body: None,
        }
    }
}

impl GetRouteWithReqAndRes {
    pub fn new() -> GetRouteWithReqAndRes {
        Default::default()
    }
    pub fn set_path(mut self, path: &'static str) -> Self {
        self.path = Some(path);
        self
    }
    pub fn set_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
    pub fn set_wildcard(mut self, wildcard: String) -> Self {
        self.wildcard = Some(wildcard);
        self
    }
    pub fn set_body(mut self, body: fn(Request) -> Response) -> Self {
        self.get_body = Some(body);
        self
    }

    pub fn get_body(&self) -> Option<fn(Request) -> Response> {
        self.get_body
    }
}

impl ToResponse for GetRouteWithReqAndRes {
    fn to_res(&self, res: Request) -> Response {
        self.get_body().unwrap()(res)
    }
}

impl Route for GetRouteWithReqAndRes {
    fn clone_dyn(&self) -> Box<dyn Route> {
        Box::new(self.clone())
    }
    fn get_method(&self) -> Method {
        self.method
    }
    fn get_path(&self) -> &str {
        self.path.unwrap()
    }
    fn wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
    #[cfg(test)]
    fn any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct BasicPostRoute {
    path: Option<&'static str>,
    method: Method,
    wildcard: Option<String>,
    is_args: Option<bool>,
    post_body: Option<fn() -> Vec<u8>>,
    post_body_with: Option<fn(Request) -> Vec<u8>>,
    post_body_with_res: Option<fn(Request) -> Response>,
    is_ret_res: bool,
}

impl Default for BasicPostRoute {
    fn default() -> Self {
        BasicPostRoute {
            path: None,
            method: Method::POST,
            wildcard: None,
            is_args: None,
            post_body: None,
            post_body_with: None,
            post_body_with_res: None,
            is_ret_res: false,
        }
    }
}

impl BasicPostRoute {
    pub fn new() -> BasicPostRoute {
        Default::default()
    }
    pub fn set_path(mut self, path: &'static str) -> Self {
        self.path = Some(path);
        self
    }
    pub fn set_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
    pub fn set_wildcard(mut self, wildcard: String) -> Self {
        self.wildcard = Some(wildcard);
        self
    }
    pub fn set_is_args(mut self, is_args: bool) -> Self {
        self.is_args = Some(is_args);
        self
    }
    pub fn set_body(mut self, body: fn() -> Vec<u8>) -> Self {
        self.post_body = Some(body);
        self
    }
    pub fn set_body_with(mut self, body: fn(Request) -> Vec<u8>) -> Self {
        self.post_body_with = Some(body);
        self
    }
    pub fn set_body_with_res(mut self, body: fn(Request) -> Response) -> Self {
        self.post_body_with_res = Some(body);
        self
    }
    pub fn set_is_ret_res(mut self, is_ret_res: bool) -> Self {
        self.is_ret_res = is_ret_res;
        self
    }
}

impl ToResponse for BasicPostRoute {
    fn to_res(&self, _req: Request) -> Response {
        Response::new()
            .body(self.post_body.unwrap()())
            .mime("text/plain")
            .status_line("HTTP/1.1 200 OK\r\n")
    }
}

impl Route for BasicPostRoute {
    fn get_path(&self) -> &str {
        self.path.unwrap()
    }
    fn get_method(&self) -> Method {
        self.method
    }
    fn wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
    fn clone_dyn(&self) -> Box<dyn Route> {
        Box::new(self.clone())
    }
    #[cfg(test)]
    fn any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct PostRouteWithReq {
    path: Option<&'static str>,
    method: Method,
    wildcard: Option<String>,
    post_body: Option<fn(Request) -> Vec<u8>>,
}
impl Default for PostRouteWithReq {
    fn default() -> Self {
        PostRouteWithReq {
            path: None,
            method: Method::POST,
            wildcard: None,
            post_body: None,
        }
    }
}
impl PostRouteWithReq {
    pub fn new() -> PostRouteWithReq {
        Default::default()
    }

    pub fn set_path(mut self, path: &'static str) -> Self {
        self.path = Some(path);
        self
    }
    pub fn set_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
    pub fn set_wildcard(mut self, wildcard: String) -> Self {
        self.wildcard = Some(wildcard);
        self
    }

    pub fn set_body(mut self, body: fn(Request) -> Vec<u8>) -> Self {
        self.post_body = Some(body);
        self
    }
}
impl ToResponse for PostRouteWithReq {
    fn to_res(&self, req: Request) -> Response {
        Response::new()
            .body(self.post_body.unwrap()(req))
            .mime("text/plain")
            .status_line("HTTP/1.1 200 OK\r\n")
    }
}

impl Route for PostRouteWithReq {
    fn clone_dyn(&self) -> Box<dyn Route> {
        Box::new(self.clone())
    }
    fn get_method(&self) -> Method {
        self.method
    }
    fn get_path(&self) -> &str {
        self.path.unwrap()
    }
    fn wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
    #[cfg(test)]
    fn any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct PostRouteWithReqAndRes {
    path: Option<&'static str>,
    method: Method,
    wildcard: Option<String>,
    post_body: Option<fn(Request) -> Response>,
}
impl Default for PostRouteWithReqAndRes {
    fn default() -> Self {
        PostRouteWithReqAndRes {
            path: None,
            method: Method::POST,
            wildcard: None,
            post_body: None,
        }
    }
}
impl PostRouteWithReqAndRes {
    pub fn new() -> PostRouteWithReqAndRes {
        Default::default()
    }

    pub fn set_path(mut self, path: &'static str) -> Self {
        self.path = Some(path);
        self
    }
    pub fn set_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
    pub fn set_wildcard(mut self, wildcard: String) -> Self {
        self.wildcard = Some(wildcard);
        self
    }

    pub fn set_body(mut self, body: fn(Request) -> Response) -> Self {
        self.post_body = Some(body);
        self
    }
}
impl ToResponse for PostRouteWithReqAndRes {
    fn to_res(&self, req: Request) -> Response {
        self.post_body.unwrap()(req)
    }
}

impl Route for PostRouteWithReqAndRes {
    fn clone_dyn(&self) -> Box<dyn Route> {
        Box::new(self.clone())
    }
    fn get_method(&self) -> Method {
        self.method
    }
    fn get_path(&self) -> &str {
        self.path.unwrap()
    }
    fn wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
    #[cfg(test)]
    fn any(&self) -> &dyn Any {
        self
    }
}
