use std::{
    net::{Incoming, TcpListener},
    sync::Arc,
};

use crate::{http::start_http, request::Request, thread_pool::ThreadPool};
pub use dyn_clone::DynClone;

#[cfg(feature = "ssl")]
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[derive(Clone, Copy)]
pub enum Method {
    GET,
    POST,
}
pub trait Route: DynClone {
    fn get_path(&self) -> &str;
    fn get_method(&self) -> Method;
    fn get_body(&self) -> fn(Request) -> Vec<u8>;
    fn post_body(&self) -> fn(Request) -> Vec<u8>;
    fn wildcard(&self) -> Option<String>;
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
    pub fn new(socket: TcpListener, config: Config) -> HttpListener {
        #[cfg(feature = "log")]
        log::debug!("Using {} threads", num_cpus::get());

        if config.ssl {
            #[cfg(feature = "ssl")]
            let ssl_acpt = Some(build_https(
                config.ssl_chain.clone().unwrap(),
                config.ssl_priv.clone().unwrap(),
            ));
            return HttpListener {
                socket,
                config,
                pool: ThreadPool::new(num_cpus::get()),
                #[cfg(feature = "ssl")]
                ssl_acpt,
                use_pool: true,
            };
        } else {
            return HttpListener {
                socket,
                config,
                pool: ThreadPool::new(num_cpus::get()),
                #[cfg(feature = "ssl")]
                ssl_acpt: None,
                use_pool: true,
            };
        }
    }

    pub fn threads(mut self, threads: usize) -> Self {
        self.pool = ThreadPool::new(threads);
        self
    }

    pub fn use_tp(mut self, r: bool) -> Self {
        self.use_pool = r;
        self
    }

    pub fn start(self) {
        start_http(self)
    }

    pub fn get_stream(&self) -> Incoming<'_> {
        self.socket.incoming()
    }
}

pub struct Routes {
    routes: Vec<Box<dyn Route>>,
}

impl Routes {
    pub fn new<R: Into<Vec<Box<dyn Route>>>>(routes: R) -> Routes {
        let routes = routes.into();
        Routes { routes }
    }

    pub fn get_stream(self) -> Vec<Box<dyn Route>> {
        self.routes
    }
}

#[derive(Clone)]
pub struct Config {
    mount_point: Option<String>,
    get_routes: Option<Vec<(String, fn(Request) -> Vec<u8>, Option<String>)>>,
    post_routes: Option<Vec<(String, fn(Request) -> Vec<u8>, Option<String>)>>,
    debug: bool,
    pub ssl: bool,
    ssl_chain: Option<String>,
    ssl_priv: Option<String>,
    headers: Option<Vec<String>>,
    br: bool,
    gzip: bool,
    spa: bool,
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

        #[cfg(feature = "log")]
        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Warn)
            .env()
            .init()
            .unwrap();

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
    /// ```no_run
    /// use tinyhttp::prelude::*;
    ///
    ///
    /// #[get("/test")]
    /// fn get_test() -> &'static str {
    ///   "Hello, World!"
    /// }
    ///
    /// #[post("/test")]
    /// fn post_test(body: Request) -> Vec<u8> {
    ///   "Hello, Post!".into()
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
        let mut get_routes: Vec<(String, fn(Request) -> Vec<u8>, Option<String>)> = vec![];
        let mut post_routes: Vec<(String, fn(Request) -> Vec<u8>, Option<String>)> = vec![];
        let routes = routes.get_stream();

        for route in routes {
            let clone = dyn_clone::clone_box(&*route);
            match route.get_method() {
                Method::GET => {
                    #[cfg(feature = "log")]
                    log::info!("GET Route init!: {}", clone.get_path());
                    get_routes.push((
                        clone.get_path().to_string(),
                        clone.get_body(),
                        clone.wildcard(),
                    ));
                }
                Method::POST => {
                    #[cfg(feature = "log")]
                    log::info!("POST Route init!: {}", clone.get_path());
                    post_routes.push((
                        clone.get_path().to_string(),
                        clone.post_body(),
                        clone.wildcard(),
                    ));
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
    pub fn headers<P: Into<Vec<String>>>(mut self, headers: P) -> Self {
        self.headers = Some(headers.into());
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
    pub fn get_headers(&self) -> Option<Vec<String>> {
        match self.headers.clone() {
            Some(vec) => Some(vec),
            None => None,
        }
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
    pub fn get_mount(&self) -> Option<String> {
        self.mount_point.clone()
    }
    pub fn get_routes(
        &self,
        path: String,
    ) -> Option<(String, fn(Request) -> Vec<u8>, Option<String>)> {
        match self.get_routes.clone() {
            Some(vec) => {
                for i in vec {
                    if self.get_debug() {
                        #[cfg(feature = "log")]
                        log::info!("Route found: {}", i.0);
                    }
                    if i.0 == path {
                        return Some(i);
                    } else if path.contains(&i.0) && i.2.is_some() {
                        let split = path.split(&i.0).last().unwrap();

                        return Some((i.0.clone(), i.1, Some(split.to_string())));
                    } else {
                        return None;
                    }
                }
            }
            None => return None,
        }
        None
    }

    pub fn post_routes(&self) -> Option<Vec<(String, fn(Request) -> Vec<u8>, Option<String>)>> {
        self.post_routes.clone()
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
