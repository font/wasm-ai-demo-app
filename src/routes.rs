use tera::{Context, Tera};
use warp::{Filter, Reply};

// Define static variables for HTML templates
static BASE_TEMPLATE: &str = include_str!("templates/base.html");
static INDEX_TEMPLATE: &str = include_str!("templates/index.html");
static INFERENCE_TEMPLATE: &str = include_str!("templates/inference.html");
static NOT_FOUND_TEMPLATE: &str = include_str!("templates/404.html");

fn render_template(template_name: &str) -> String {
    let mut tera = Tera::default();
    // Add the templates to the Tera instance
    tera.add_raw_template("base.html", BASE_TEMPLATE).unwrap();
    tera.add_raw_template("index.html", INDEX_TEMPLATE).unwrap();
    tera.add_raw_template("inference.html", INFERENCE_TEMPLATE)
        .unwrap();
    tera.add_raw_template("404.html", NOT_FOUND_TEMPLATE)
        .unwrap();

    let context = Context::new();
    tera.render(template_name, &context)
        .expect("Failed to render Tera template.")
}

pub fn root() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::path::end()
        .map(|| {
            let body = render_template("index.html");
            warp::reply::html(body)
        })
        .boxed()
}

pub fn inference() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::path!("inference")
        .and(warp::post())
        .map(|| {
            let body = render_template("inference.html");
            warp::reply::html(body)
        })
        .boxed()
}

pub fn not_found() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::any()
        .map(|| {
            let body = render_template("404.html");
            warp::reply::with_status(warp::reply::html(body), warp::http::StatusCode::NOT_FOUND)
        })
        .boxed()
}
