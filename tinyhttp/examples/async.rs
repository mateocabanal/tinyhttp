use tinyhttp::prelude::*;

use tokio::net::TcpListener;

#[get("/")]
fn get() -> &'static str {
    "Hello World!\n"
}

#[tokio::main]
async fn main() {
    let socket = TcpListener::bind(":::9001").await.unwrap();
    let config = Config::new().routes(Routes::new(vec![get()]));
    HttpListener::new(socket, config).start().await;
}
