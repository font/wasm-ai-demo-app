use serde_json::json;
use std::fs;
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

const UPLOADED_IMAGE_NAME: &str = "uploaded_image.jpg";

pub fn inference() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::path!("inference")
        .and(warp::post())
        .and(warp::body::bytes())
        .map(|body: warp::hyper::body::Bytes| {
            let inference_template = render_template("inference.html");

            // Process the raw image data here
            match process_image(body) {
                Ok(_) => warp::reply::json(&json!({
                    // Return a JSON response with the rendered template and results
                    "message": "Image received and processed successfully",
                    "template": inference_template,
                })),
                Err(err) => warp::reply::json(&json!({
                    "status": "error",
                    "message": format!("Error processing image: {}", err),
                    "template": inference_template,
                })),
            }
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

fn process_image(image_data: warp::hyper::body::Bytes) -> Result<String, String> {
    // Save the image data to the file
    if let Err(err) = fs::write(UPLOADED_IMAGE_NAME, image_data.as_ref()) {
        Err(format!("Failed to save image: {}", err))
    } else {
        println!("Image saved to: {}", UPLOADED_IMAGE_NAME);
        let results = crate::inference::infer_image(UPLOADED_IMAGE_NAME);
        Ok(results)
    }
}
