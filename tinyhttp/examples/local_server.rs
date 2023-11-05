use tinyhttp::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;
    let sock = std::net::TcpListener::bind("0.0.0.0:42070")?;
    let config = Config::new().mount_point("./");
    let http = HttpListener::new(sock, config);
    http.start();

    Ok(())
}
