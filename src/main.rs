use wasmedge_wasi_socket::TcpListener;
mod http;

fn main() -> std::io::Result<()> {
    let host = "0.0.0.0";
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let listener = TcpListener::bind(format!("{}:{}", host, port), false)?;
    println!("Now listening at {}:{}", host, port);
    loop {
        let _ = http::handle_client(listener.accept(false)?.0);
    }
}
