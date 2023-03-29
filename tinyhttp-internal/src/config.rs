use std::{collections::HashMap, sync::Arc};

use crate::request::Request;
pub use dyn_clone::DynClone;
use std::fmt::Debug;

#[cfg(feature = "ssl")]
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[cfg(feature = "async")]
use crate::async_http::start_http;

#[cfg(feature = "async")]
use async_std::net::{Incoming, TcpListener};

#[cfg(feature = "async")]
use futures::executor::{ThreadPool, ThreadPoolBuilder};

#[cfg(not(feature = "async"))]
use crate::http::start_http;

use crate::response::Response;

use rusty_pool::{Builder, ThreadPool};

#[cfg(not(feature = "async"))]
use std::net::{Incoming, TcpListener};

#[cfg(test)]
use std::any::Any;

type RouteVec = Vec<Box<dyn Route>>;

#[derive(Clone, Copy, Debug)]
pub enum Method {
    GET,
    POST,
}

pub trait ToResponse: DynClone + Sync + Send + Debug {
    fn to_res(&self, res: Request) -> Response;
}

pub trait Route: DynClone + Sync + Send + Debug + ToResponse {
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
    socket: TcpListener,
    pub config: Config,
    pub pool: ThreadPool,
    pub use_pool: bool,
    #[cfg(feature = "ssl")]
    pub ssl_acpt: Option<Arc<SslAcceptor>>,
}

impl HttpListener {
    pub fn new<P: Into<TcpListener>>(socket: P, config: Config) -> HttpListener {
        #[cfg(feature = "log")]
        log::debug!("Using {} threads", num_cpus::get());

        if config.ssl {
            #[cfg(feature = "ssl")]
            let ssl_acpt = Some(build_https(
                config.ssl_chain.clone().unwrap(),
                config.ssl_priv.clone().unwrap(),
            ));
            HttpListener {
                socket: socket.into(),
                config,
                #[cfg(not(feature = "async"))]
                pool: ThreadPool::default(),
                #[cfg(feature = "async")]
                pool: ThreadPoolBuilder::new()
                    .pool_size(num_cpus::get())
                    .create()
                    .unwrap(),
                #[cfg(feature = "ssl")]
                ssl_acpt,
                use_pool: true,
            }
        } else {
            HttpListener {
                socket: socket.into(),
                config,
                #[cfg(not(feature = "async"))]
                pool: ThreadPool::default(),
                #[cfg(feature = "async")]
                pool: ThreadPoolBuilder::new()
                    .pool_size(num_cpus::get())
                    .create()
                    .unwrap(),
                #[cfg(feature = "ssl")]
                ssl_acpt: None,
                use_pool: true,
            }
        }
    }

    pub fn threads(mut self, threads: usize) -> Self {
        #[cfg(not(feature = "async"))]
        let pool = Builder::new().core_size(threads).build();

        #[cfg(feature = "async")]
        let pool = ThreadPoolBuilder::new()
            .pool_size(threads)
            .create()
            .unwrap();

        self.pool = pool;
        self
    }

    pub fn use_tp(mut self, r: bool) -> Self {
        self.use_pool = r;
        self
    }

    #[cfg(not(feature = "async"))]
    pub fn start(self) {
        start_http(self);
    }

    #[cfg(feature = "async")]
    pub async fn start_async(self) {
        start_http(self).await;
    }

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
    get_routes: Option<RouteVec>,
    post_routes: Option<RouteVec>,
    debug: bool,
    pub ssl: bool,
    ssl_chain: Option<String>,
    ssl_priv: Option<String>,
    headers: Option<HashMap<String, String>>,
    br: bool,
    gzip: bool,
    spa: bool,
    http2: bool,
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
    /// use tinyhttp::internal::config::*;
    /// use tinyhttp::codegen::*;
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
            br: false,
            spa: false,
            http2: false,
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
        let mut get_routes: RouteVec = vec![];
        let mut post_routes: RouteVec = vec![];
        let routes = routes.get_stream();

        for route in routes {
            let clone = dyn_clone::clone_box(&*route);
            match route.get_method() {
                Method::GET => {
                    #[cfg(feature = "log")]
                    log::info!("GET Route init!: {}", clone.get_path());
                    get_routes.push(route);
                }
                Method::POST => {
                    #[cfg(feature = "log")]
                    log::info!("POST Route init!: {}", clone.get_path());
                    post_routes.push(route);
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

    /// DOES NOT WORK!
    /// Enables brotli compression
    pub fn br(mut self, res: bool) -> Self {
        self.br = res;
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


    pub fn get_headers(&self) -> Option<&HashMap<String, String>> {
        self.headers.as_ref()
    }
    pub fn get_br(&self) -> bool {
        self.br
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
    pub fn get_routes(&self, path: &mut String) -> Option<Box<dyn Route>> {
        if path.ends_with('/') && path.matches('/').count() > 1 {
            path.pop();
        };

        #[cfg(feature = "log")]
        log::trace!("get_routes -> new_path: {}", &path);

        match self.get_routes.clone() {
            Some(routes) => {
                for route in routes {
                    #[cfg(feature = "log")]
                    log::trace!("get_routes -> paths: {}", route.get_path());

                    if route.get_path() == path {
                        #[cfg(feature = "log")]
                        log::trace!("Route found: {:#?}", route);

                        return Some(route);
                    } else if path.contains(route.get_path()) && route.wildcard().is_some() {
                        return Some(route);
                    } else {
                        continue;
                    }
                }
            }
            None => return None,
        }
        None
    }

    pub fn post_routes(&self, path: &mut String) -> Option<Box<dyn Route>> {
        #[cfg(feature = "log")]
        log::trace!("post_routes -> path: {}", path);
        if path.ends_with('/') && path.matches('/').count() > 1 {
            path.pop();
        };
        #[cfg(feature = "log")]
        log::trace!("get_routes -> new_path: {}", &path);

        match self.post_routes.clone() {
            Some(routes) => {
                for route in routes {
                    if route.get_path() == path {
                        /*#[cfg(feature = "log")]
                        log::trace!(
                            "POST route found: {:#?}, get_body: {:?}, get_body_with: {:?}",
                            route,
                          //  route.post_body().is_some(),
                          //  route.post_body_with().is_some()
                        );*/
                        #[cfg(feature = "log")]
                        log::trace!("PATH: {}", path);

                        return Some(route);
                    } else if path.contains(route.get_path()) && route.wildcard().is_some() {
                        return Some(route);
                    } else {
                        continue;
                    }
                }
            }
            None => return None,
        }
        None
    }

    pub fn get_spa(&self) -> bool {
        self.spa
    }
}

#[cfg(feature = "ssl")]
pub fn build_https(chain: String, private: String) -> Arc<SslAcceptor> {
    let mut acceptor = SslAcceptor::mozilla_modern_v5(SslMethod::tls()).unwrap();
    acceptor.set_certificate_chain_file(chain).unwrap();
    acceptor
        .set_private_key_file(private, SslFiletype::PEM)
        .unwrap();
    acceptor.check_private_key().unwrap();
    Arc::new(acceptor.build())
}
