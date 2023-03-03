use wasmedge_wasi_socket::TcpListener;
mod http;

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    println!("new connection at {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port), false)?;
    loop {
        let _ = http::handle_client(listener.accept(false)?.0);
    }
}
