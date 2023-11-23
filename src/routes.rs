use lazy_static::lazy_static;
use serde_json::json;
use std::fs;
use tera::{Context, Tera};
use warp::{Filter, Reply};

// Define static variables for HTML templates
static BASE_TEMPLATE: &str = include_str!("templates/base.html");
static INDEX_TEMPLATE: &str = include_str!("templates/index.html");
static INFERENCE_TEMPLATE: &str = include_str!("templates/inference.html");
static NOT_FOUND_TEMPLATE: &str = include_str!("templates/404.html");

// Define a lazy-static variable to store the Tera instance
lazy_static! {
    static ref TERA: Tera = {
        let mut tera = Tera::default();

        // Add the templates to the Tera instance
        tera.add_raw_template("base.html", BASE_TEMPLATE).unwrap();
        tera.add_raw_template("index.html", INDEX_TEMPLATE).unwrap();
        tera.add_raw_template("inference.html", INFERENCE_TEMPLATE)
            .unwrap();
        tera.add_raw_template("404.html", NOT_FOUND_TEMPLATE)
            .unwrap();
        tera
    };
}

fn render_template(template_name: &str) -> Result<String, String> {
    let context = Context::new();
    render_template_context(template_name, &context)
}

fn render_template_context(template_name: &str, context: &tera::Context) -> Result<String, String> {
    match TERA.render(template_name, &context) {
        Ok(rendered) => Ok(rendered),
        Err(e) => Err(format!("Error rendering template: {}", e)),
    }
}

pub fn root() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::path::end()
        .map(|| {
            // Render the index template
            let response = match render_template("index.html") {
                Ok(index_template) => {
                    // Construct a JSON response with the rendered template
                    json!({
                        "status": "success",
                        "message": "Welcome to the image inference API",
                        "template": index_template,
                    })
                }
                Err(err) => {
                    // Construct an error JSON response with the template rendering error
                    json!({
                        "status": "error",
                        "message": format!("Error rendering template: {}", err),
                        "template": null,
                    })
                }
            };
            // Return the JSON response
            warp::reply::json(&response)
        })
        .boxed()
}

const UPLOADED_IMAGE_NAME: &str = "uploaded_image.jpg";

pub fn inference() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::path!("inference")
        .and(warp::post())
        .and(warp::body::bytes())
        .map(|body: warp::hyper::body::Bytes| {
            // Process the raw image data here
            let response = match process_image(body) {
                Ok(results) => {
                    let mut context = Context::new();
                    context.insert("path_to_image", UPLOADED_IMAGE_NAME);
                    context.insert("inference_result", &results);

                    match render_template_context("inference.html", &context) {
                        Ok(inference_template) => {
                            json!({
                            // Return a JSON response with the rendered template and results
                            "status": "success",
                            "message": "Image received and processed successfully",
                            "template": inference_template,
                            })
                        }
                        Err(err) => {
                            json!({
                                "status": "error",
                                "message": format!("Error rendering template: {}", err),
                                "template": null,
                            })
                        }
                    }
                }
                Err(err) => {
                    json!({
                        "status": "error",
                        "message": format!("Error processing image: {}", err),
                        "template": null,
                    })
                }
            };
            // Return the JSON response
            warp::reply::json(&response)
        })
        .boxed()
}

pub fn not_found() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::any()
        .map(|| {
            // Render the 404 template
            let response = match render_template("404.html") {
                Ok(not_found_template) => {
                    // Construct a JSON response with the rendered template
                    json!({
                        "status": "error",
                        "message": "Not found",
                        "template": not_found_template,
                    })
                }
                Err(err) => {
                    // Construct an error JSON response with the template rendering error
                    json!({
                        "status": "error",
                        "message": format!("Error rendering 404 template: {}", err),
                        "template": null,
                    })
                }
            };
            // Return the JSON response
            warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::NOT_FOUND,
            )
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
