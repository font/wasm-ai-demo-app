use std::net::SocketAddr;
use warp::Filter;
mod routes;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    // Combine the routes from the routes module
    let routes = routes::root()
        .or(routes::inference())
        .or(routes::not_found());

    println!("Listening on http://{}/", addr);

    // Start the warp server
    warp::serve(routes).run(addr).await;
}
