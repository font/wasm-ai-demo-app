use lazy_static::lazy_static;
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
                    // Return an HTML response with the rendered template
                    warp::reply::with_status(
                        warp::reply::html(index_template),
                        warp::http::StatusCode::OK,
                    )
                }
                Err(err) => {
                    // Return an error HTML response with the template rendering error
                    warp::reply::with_status(
                        warp::reply::html(format!(
                            "<h1>Error rendering index template: {}</h1>",
                            err
                        )),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            };
            // Return the HTML response
            response
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
                            // Return an HTML response with the rendered template
                            warp::reply::with_status(
                                warp::reply::html(inference_template),
                                warp::http::StatusCode::OK,
                            )
                        }
                        Err(err) => {
                            // Return an error HTML response with the template rendering error
                            warp::reply::with_status(
                                warp::reply::html(format!(
                                    "<h1>Error rendering inference template: {}</h1>",
                                    err
                                )),
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            )
                        }
                    }
                }
                Err(err) => {
                    // Return an error HTML response with the inference error
                    warp::reply::with_status(
                        warp::reply::html(format!("<h1>Error processing image: {}</h1>", err)),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            };
            // Return the HTML response
            response
        })
        .boxed()
}

pub fn upload() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::path!("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(5 * 1024 * 1024)) // Set a 5MB limit on the multipart form data
        .and_then(|form: warp::multipart::FormData| {
            // Iterate over the form fields
            let result: Result<Vec<_>, _> = form
                .and_then(|field| async move {
                    // Check if the field is a file
                    if let Some(filename) = field.filename() {
                        // Check if the file is a JPEG image
                        if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                            // Process the raw image data here
                            match process_image(field.into_inner()) {
                                Ok(results) => {
                                    let mut context = Context::new();
                                    context.insert("path_to_image", UPLOADED_IMAGE_NAME);
                                    context.insert("inference_result", &results);

                                    match render_template_context("inference.html", &context) {
                                        Ok(inference_template) => {
                                            // Return an HTML response with the rendered template
                                            Ok(warp::reply::with_status(
                                                warp::reply::html(inference_template),
                                                warp::http::StatusCode::OK,
                                            ))
                                        }
                                        Err(err) => {
                                            // Return an error HTML response with the template rendering error
                                            Ok(warp::reply::with_status(
                                                warp::reply::html(format!(
                                                    "<h1>Error rendering inference template: {}</h1>",
                                                    err
                                                )),
                                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                            ))
                                        }
                                    }
                                }
                                Err(err) => {
                                    // Return an error HTML response with the inference error
                                    Ok(warp::reply::with_status(
                                        warp::reply::html(format!(
                                            "<h1>Error processing image: {}</h1>",
                                            err
                                        )),
                                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                    ))
                                }
                            }
                            // Return the field if it is a JPEG image
                        } else {
                            // Return an error if the file is not a JPEG image
                            return Err(warp::reject::custom(UploadError::NotAJpegImage));
                        }
                    } else {
                        // Return an error if the field is not a file
                        return Err(warp::reject::custom(UploadError::NotAFile));
                    }
                })
                .try_collect();
            // Return the HTML response
            warp::reply::html(result.unwrap_or_else(|_| {
                warp::reply::html("<h1>Error processing form data</h1>")
            }))
        })
        .boxed()
}

pub fn not_found() -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::any()
        .map(|| {
            // Render the 404 template
            let response = match render_template("404.html") {
                Ok(not_found_template) => {
                    // Return an HTML response with the rendered template
                    warp::reply::with_status(
                        warp::reply::html(not_found_template),
                        warp::http::StatusCode::NOT_FOUND,
                    )
                }
                Err(err) => {
                    // Return an error HTML response with the template rendering error
                    warp::reply::with_status(
                        warp::reply::html(format!(
                            "<h1>Error rendering 404 template: {}</h1>",
                            err
                        )),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            };
            // Return the HTML response
            response
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
