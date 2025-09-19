use std::{
    collections::HashMap,
    net::TcpStream,
    ops::Deref,
    sync::OnceLock,
};

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

#[derive(Clone, Copy, Debug)]
pub enum Method {
    GET,
    POST,
}

pub trait ToResponse: DynClone + Sync + Send {
    fn to_res(&self, res: Request, sock: &mut TcpStream) -> Response;
}

pub trait Route: DynClone + Sync + Send + ToResponse {
    fn get_path(&self) -> &str;
    fn get_method(&self) -> Method;
    fn wildcard(&self) -> Option<String>;
    fn clone_dyn(&self) -> Box<dyn Route>;

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
    /// Generates default settings (which don't work by itself)
    ///
    /// Chain with mount_point or routes
    ///
    /// ### Example:
    /// ```ignore
    /// use tinyhttp::prelude::*;
    ///
    /// #[get("/test")]
    /// fn get_test() -> String {
    ///   String::from("Hello, there!\n")
    /// }
    ///
    /// let routes = Routes::new(vec![get_test()]);
    /// let routes_config = Config::new().routes(routes);
    /// /// or
    /// let mount_config = Config::new().mount_point(".");
    /// ```

    pub fn new() -> Config {
        //assert!(routes.len() > 0);

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

    /// A mount point that will be searched when a request isn't defined with a get or post route
    ///
    /// ### Example:
    /// ```ignore
    /// let config = Config::new().mount_point(".")
    /// /// if index.html exists in current directory, it will be returned if "/" or "/index.html" is requested.
    /// ```

    pub fn mount_point<P: Into<String>>(mut self, path: P) -> Self {
        self.mount_point = Some(path.into());
        self
    }

    /// Add routes with a Route member
    ///
    /// ### Example:
    /// ```ignore
    /// use tinyhttp::prelude::*;
    ///
    ///
    /// #[get("/test")]
    /// fn get_test() -> &'static str {
    ///   "Hello, World!"
    /// }
    ///
    /// #[post("/test")]
    /// fn post_test() -> Vec<u8> {
    ///   b"Hello, Post!".to_vec()
    /// }
    ///
    /// fn main() {
    ///   let socket = TcpListener::new(":::80").unwrap();
    ///   let routes = Routes::new(vec![get_test(), post_test()]);
    ///   let config = Config::new().routes(routes);
    ///   let http = HttpListener::new(socket, config);
    ///
    ///   http.start();
    /// }
    /// ```

    pub fn routes(mut self, routes: Routes) -> Self {
        let mut get_routes = HashMap::new();
        let mut post_routes = HashMap::new();
        let routes = routes.get_stream();

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
        if !get_routes.is_empty() {
            self.get_routes = Some(get_routes);
        } else {
            self.get_routes = None;
        }

        if !post_routes.is_empty() {
            self.post_routes = Some(post_routes);
        } else {
            self.post_routes = None;
        }

        self
    }

    /// Enables SSL
    ///
    /// ### Example:
    /// ```ignore
    /// let config = Config::new().ssl("./fullchain.pem", "./privkey.pem");
    /// ```
    /// This will only accept HTTPS connections

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

    /// Define custom headers
    ///
    /// ```ignore
    /// let config = Config::new().headers(vec!["Access-Control-Allow-Origin: *".into()]);
    /// ```
    pub fn headers(mut self, headers: Vec<String>) -> Self {
        let mut hash_map: HashMap<String, String> = HashMap::new();
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

    /// Enables gzip compression
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
    pub fn get_gzip(&self) -> bool {
        self.gzip
    }
    pub fn get_debug(&self) -> bool {
        self.debug
    }
    pub fn get_mount(&self) -> Option<&String> {
        self.mount_point.as_ref()
    }
    pub fn get_routes(&self, req_path: &str) -> Option<&dyn Route> {
        let req_path = if req_path.ends_with('/') && req_path.matches('/').count() > 1 {
            let mut chars = req_path.chars();
            chars.next_back();
            chars.as_str()
        } else {
            req_path
        };

        #[cfg(feature = "log")]
        log::trace!("get_routes -> new_path: {}", &req_path);

        let routes = self.get_routes.as_ref()?;

        if let Some(route) = routes.get(req_path) {
            return Some(route.deref());
        }

        if let Some((_, wildcard_route)) = routes
            .iter()
            .find(|(path, route)| req_path.starts_with(*path) && route.wildcard().is_some())
        {
            return Some(wildcard_route.deref());
        }

        None
    }

    pub fn post_routes(&self, req_path: &str) -> Option<&dyn Route> {
        #[cfg(feature = "log")]
        log::trace!("post_routes -> path: {}", req_path);

        let req_path = if req_path.ends_with('/') && req_path.matches('/').count() > 1 {
            let mut chars = req_path.chars();
            chars.next_back();
            chars.as_str()
        } else {
            req_path
        };

        #[cfg(feature = "log")]
        log::trace!("get_routes -> new_path: {}", &req_path);

        let routes = self.post_routes.as_ref()?;

        if let Some(route) = routes.get(req_path) {
            return Some(route.deref());
        }

        if let Some((_, wildcard_route)) = routes
            .iter()
            .find(|(path, route)| req_path.starts_with(*path) && route.wildcard().is_some())
        {
            return Some(wildcard_route.deref());
        }

        None
    }

    pub fn get_spa(&self) -> bool {
        self.spa
    }
}
