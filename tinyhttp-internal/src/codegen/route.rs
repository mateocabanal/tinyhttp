use crate::config::{Method, Route};
use crate::request::Request;
use crate::response::Response;

#[derive(Clone, Debug)]
pub struct GetRoute {
    path: Option<&'static str>,
    method: Method,
    wildcard: Option<String>,
    is_args: Option<bool>,
    get_body: Option<fn() -> Vec<u8>>,
    get_body_with: Option<fn(Request) -> Vec<u8>>,

    get_body_with_res: Option<fn(Request) -> Response>,
    is_ret_res: bool,
}

impl Default for GetRoute {
    fn default() -> Self {
        GetRoute {
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

impl GetRoute {
    pub fn new() -> GetRoute {
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

impl Route for GetRoute {
    fn get_path(&self) -> &str {
        self.path.unwrap()
    }
    fn get_method(&self) -> Method {
        self.method
    }
    fn get_body(&self) -> Option<fn() -> Vec<u8>> {
        self.get_body
    }
    fn get_body_with(&self) -> Option<fn(Request) -> Vec<u8>> {
        self.get_body_with
    }
    fn get_body_with_res(&self) -> Option<fn(Request) -> Response> {
        self.get_body_with_res
    }
    fn post_body(&self) -> Option<fn() -> Vec<u8>> {
        None
    }
    fn post_body_with(&self) -> Option<fn(Request) -> Vec<u8>> {
        None
    }
    fn post_body_with_res(&self) -> Option<fn(Request) -> Response> {
        None
    }
    fn wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
    fn is_args(&self) -> bool {
        self.is_args.unwrap()
    }
    fn is_ret_res(&self) -> bool {
        self.is_ret_res
    }
    fn clone_dyn(&self) -> Box<dyn Route> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct PostRoute {
    path: Option<&'static str>,
    method: Method,
    wildcard: Option<String>,
    is_args: Option<bool>,
    post_body: Option<fn() -> Vec<u8>>,
    post_body_with: Option<fn(Request) -> Vec<u8>>,
    post_body_with_res: Option<fn(Request) -> Response>,
    is_ret_res: bool,
}

impl Default for PostRoute {
    fn default() -> Self {
        PostRoute {
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

impl PostRoute {
    pub fn new() -> PostRoute {
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

impl Route for PostRoute {
    fn get_path(&self) -> &str {
        self.path.unwrap()
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
    fn get_body_with_res(&self) -> Option<fn(Request) -> Response> {
        None
    }
    fn post_body(&self) -> Option<fn() -> Vec<u8>> {
        self.post_body
    }
    fn post_body_with(&self) -> Option<fn(Request) -> Vec<u8>> {
        self.post_body_with
    }
    fn post_body_with_res(&self) -> Option<fn(Request) -> Response> {
        self.post_body_with_res
    }
    fn wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
    fn is_args(&self) -> bool {
        self.is_args.unwrap()
    }
    fn is_ret_res(&self) -> bool {
        self.is_ret_res
    }
    fn clone_dyn(&self) -> Box<dyn Route> {
        Box::new(self.clone())
    }
}
