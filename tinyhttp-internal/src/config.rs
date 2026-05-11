use std::{collections::HashMap, net::TcpStream, ops::Deref, sync::OnceLock};

use crate::{middleware::MiddlewareResponse, request::Request};
pub use dyn_clone::DynClone;
use std::fmt::Debug;

use crate::response::Response;

use rusty_pool::{Builder, ThreadPool};

#[cfg(not(feature = "async"))]
use std::net::{Incoming, TcpListener};

#[cfg(not(feature = "async"))]
use crate::http::start_http;

#[cfg(test)]
use std::any::Any;

type RouteVec = Vec<Box<dyn Route>>;

type MiddlewareFn = fn(&mut Request) -> MiddlewareResponse;

pub static PRE_MIDDLEWARE_CONST: OnceLock<Box<dyn FnMut(&mut Request) + Send + Sync>> =
    OnceLock::new();

pub static POST_MIDDLEWARE_CONST: OnceLock<Box<dyn FnMut(&mut Request) + Send + Sync>> =
    OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
}

impl Method {
    pub fn from_str(method: &str) -> Method {
        match method {
            "GET" => Method::GET,
            "POST" => Method::POST,
            _ => Method::GET,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
        }
    }
}

pub trait ToResponse: DynClone + Sync + Send {
    fn to_res(&self, res: Request, sock: &mut TcpStream) -> Response;
}

pub trait Route: DynClone + Sync + Send + ToResponse {
    fn get_path(&self) -> &str;
    fn get_method(&self) -> Method;
    fn wildcard(&self) -> Option<&str>;
    fn clone_dyn(&self) -> Box<dyn Route>;

    fn needs_request(&self) -> bool {
        true
    }

    fn cached_response(&self) -> Option<&[u8]> {
        None
    }

    #[cfg(test)]
    fn any(&self) -> &dyn Any;
}

impl Clone for Box<dyn Route> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

pub struct HttpListener {
    pub(crate) socket: TcpListener,
    pub config: Config,
    pub pool: ThreadPool,
    pub use_pool: bool,
}

impl HttpListener {
    pub fn new<P: Into<TcpListener>>(socket: P, config: Config) -> HttpListener {
        #[cfg(feature = "log")]
        log::debug!("Using {} threads", num_cpus::get());

        HttpListener {
            socket: socket.into(),
            config,
            pool: ThreadPool::default(),
            use_pool: true,
        }
    }

    pub fn threads(mut self, threads: usize) -> Self {
        let pool = Builder::new().core_size(threads).build();

        self.pool = pool;
        self
    }

    pub fn use_tp(mut self, r: bool) -> Self {
        self.use_pool = r;
        self
    }

    #[cfg(not(feature = "async"))]
    pub fn start(self) {
        let conf_clone = self.config.clone();
        start_http(self, conf_clone);
    }

    #[cfg(not(feature = "async"))]
    pub fn get_stream(&self) -> Incoming<'_> {
        self.socket.incoming()
    }
}

#[derive(Clone)]
pub struct Routes {
    routes: RouteVec,
}

impl Routes {
    pub fn new<R: Into<RouteVec>>(routes: R) -> Routes {
        let routes = routes.into();
        Routes { routes }
    }

    pub fn get_stream(self) -> RouteVec {
        self.routes
    }
}

#[derive(Clone)]
pub struct Config {
    mount_point: Option<String>,
    get_routes: Option<HashMap<String, Box<dyn Route>>>,
    post_routes: Option<HashMap<String, Box<dyn Route>>>,
    debug: bool,
    pub ssl: bool,
    ssl_chain: Option<String>,
    ssl_priv: Option<String>,
    headers: Option<HashMap<String, String>>,
    gzip: bool,
    spa: bool,
    http2: bool,
    middleware: Option<Vec<MiddlewareFn>>,
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

impl Config {
    pub fn new() -> Config {
        #[cfg(feature = "log")]
        log::info!("tinyhttp version: {}", env!("CARGO_PKG_VERSION"));

        Config {
            mount_point: None,
            get_routes: None,
            post_routes: None,
            debug: false,
            ssl: false,
            ssl_chain: None,
            ssl_priv: None,
            headers: None,
            gzip: false,
            spa: false,
            http2: false,
            middleware: None,
        }
    }

    pub fn mount_point<P: Into<String>>(mut self, path: P) -> Self {
        self.mount_point = Some(path.into());
        self
    }

    pub fn routes(mut self, routes: Routes) -> Self {
        let routes = routes.get_stream();
        let mut get_routes = HashMap::with_capacity(routes.len());
        let mut post_routes = HashMap::with_capacity(routes.len());

        for route in routes {
            match route.get_method() {
                Method::GET => {
                    #[cfg(feature = "log")]
                    log::info!("GET Route init!: {}", &route.get_path());

                    get_routes.insert(route.get_path().to_string(), route);
                }
                Method::POST => {
                    #[cfg(feature = "log")]
                    log::info!("POST Route init!: {}", &route.get_path());
                    post_routes.insert(route.get_path().to_string(), route);
                }
            }
        }

        self.get_routes = (!get_routes.is_empty()).then_some(get_routes);
        self.post_routes = (!post_routes.is_empty()).then_some(post_routes);

        self
    }

    pub fn ssl(mut self, ssl_chain: String, ssl_priv: String) -> Self {
        self.ssl_chain = Some(ssl_chain);
        self.ssl_priv = Some(ssl_priv);
        self.ssl = true;
        self
    }

    pub fn debug(mut self) -> Self {
        self.debug = true;
        self
    }

    pub fn headers(mut self, headers: Vec<String>) -> Self {
        let mut hash_map: HashMap<String, String> = HashMap::with_capacity(headers.len());
        for i in headers {
            let mut split = i.split_inclusive(": ");
            hash_map.insert(
                split.next().unwrap().to_string(),
                split.next().unwrap().to_string() + "\r\n",
            );
        }

        self.headers = Some(hash_map);
        self
    }

    pub fn spa(mut self, res: bool) -> Self {
        self.spa = res;
        self
    }

    pub fn gzip(mut self, res: bool) -> Self {
        self.gzip = res;
        self
    }

    pub fn http2(mut self, res: bool) -> Self {
        self.http2 = res;
        self
    }

    pub fn middleware(mut self, middleware: Vec<MiddlewareFn>) -> Self {
        self.middleware = Some(middleware);
        self
    }

    pub fn get_middleware(&self) -> Option<&[MiddlewareFn]> {
        self.middleware.as_deref()
    }

    pub fn get_headers(&self) -> Option<&HashMap<String, String>> {
        self.headers.as_ref()
    }

    pub fn can_use_prebuilt_routes(&self) -> bool {
        self.headers.is_none() && !self.gzip
    }

    pub fn get_gzip(&self) -> bool {
        self.gzip
    }

    pub fn get_debug(&self) -> bool {
        self.debug
    }

    pub fn get_mount(&self) -> Option<&String> {
        self.mount_point.as_ref()
    }

    pub fn route_for(&self, method: Method, req_path: &str) -> Option<&dyn Route> {
        match method {
            Method::GET => self.get_routes(req_path),
            Method::POST => self.post_routes(req_path),
        }
    }

    pub fn get_routes(&self, req_path: &str) -> Option<&dyn Route> {
        #[cfg(feature = "log")]
        log::trace!("get_routes -> new_path: {}", req_path);

        self.find_route(self.get_routes.as_ref()?, req_path)
    }

    pub fn post_routes(&self, req_path: &str) -> Option<&dyn Route> {
        #[cfg(feature = "log")]
        log::trace!("post_routes -> path: {}", req_path);

        self.find_route(self.post_routes.as_ref()?, req_path)
    }

    fn find_route<'a>(
        &self,
        routes: &'a HashMap<String, Box<dyn Route>>,
        req_path: &str,
    ) -> Option<&'a dyn Route> {
        let req_path = normalize_req_path(req_path);

        if let Some(route) = routes.get(req_path) {
            return Some(route.deref());
        }

        routes
            .iter()
            .find(|(path, route)| req_path.starts_with(path.as_str()) && route.wildcard().is_some())
            .map(|(_, route)| route.deref())
    }

    pub fn get_spa(&self) -> bool {
        self.spa
    }
}

fn normalize_req_path(req_path: &str) -> &str {
    if req_path.len() > 1 && req_path.ends_with('/') {
        &req_path[..req_path.len() - 1]
    } else {
        req_path
    }
}
