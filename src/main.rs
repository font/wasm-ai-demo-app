use std::net::SocketAddr;

use hyper::server::conn::Http;
use hyper::service::service_fn;
use tokio::net::TcpListener;

mod http;
mod inference;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = Http::new()
                .serve_connection(stream, service_fn(http::http_handler))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
