use futures_util::TryStreamExt;
use lazy_static::lazy_static;
use std::fs;
use tera::{Context, Tera};
use warp::{Buf, Filter, Reply};

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
            async {
                // Iterate over the form fields
                let result: Result<Vec<_>, _> = form
                    .and_then(|part| async {
                        // Check if the field is a file
                        if let Some(filename) = part.filename() {
                            // Check if the file is a JPEG image
                            //if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                            //    // Process the raw image data here
                            //    match field.data().await {
                            //        Some(Ok(buf)) => {
                            //            match process_image(buf.copy_to_bytes(buf.remaining())) {
                            //                Ok(results) => {
                            //                    let mut context = Context::new();
                            //                    context.insert("path_to_image", UPLOADED_IMAGE_NAME);
                            //                    context.insert("inference_result", &results);

                            //                    match render_template_context("inference.html", &context) {
                            //                        Ok(inference_template) => {
                            //                            // Return an HTML response with the rendered template
                            //                            Ok(warp::reply::with_status(
                            //                                warp::reply::html(inference_template),
                            //                                warp::http::StatusCode::OK,
                            //                            ))
                            //                        }
                            //                        Err(err) => {
                            //                            // Return an error HTML response with the template rendering error
                            //                            Ok(warp::reply::with_status(
                            //                                warp::reply::html(format!(
                            //                                    "<h1>Error rendering inference template: {}</h1>",
                            //                                    err
                            //                                )),
                            //                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            //                            ))
                            //                        }
                            //                    }
                            //                }
                            //                Err(err) => {
                            //                    // Return an error HTML response with the inference error
                            //                    Ok(warp::reply::with_status(
                            //                        warp::reply::html(format!(
                            //                            "<h1>Error processing image: {}</h1>",
                            //                            err
                            //                        )),
                            //                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            //                    ))
                            //                }
                            //            }
                            //        }
                            //        Some(Err(err)) => {
                            //            // Return an error if the field data could not be read
                            //            Ok(warp::reply::with_status(
                            //                warp::reply::html(format!(
                            //                    "<h1>Error reading field data: {}</h1>",
                            //                    err
                            //                )),
                            //                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            //            ))
                            //            //return Err(warp::reject::custom(UploadError::ReadError(
                            //            //    err.to_string(),
                            //            //)));
                            //        }
                            //        //Some(Ok(buf)) => Some(Ok(warp::hyper::body::Bytes::from(
                            //        //    buf.copy_to_bytes(buf.remaining()).to_vec(),
                            //        //))),
                            //        //Some(Err(err)) => {
                            //        //    // Return an error if the field data could not be read
                            //        //    Some(Err(warp::reject::custom(UploadError::ReadError(
                            //        //        err.to_string(),
                            //        //    ))))
                            //        //}
                            //    }
                            //    // Return the field if it is a JPEG image
                            //} else {
                            //    // Return an error if the file is not a JPEG image
                            //    Ok(warp::reply::with_status(
                            //        warp::reply::html(format!("<h1>Not a JPEG image</h1>")),
                            //        warp::http::StatusCode::BAD_REQUEST,
                            //    ))
                            //    //return Err(warp::reject::custom(UploadError::NotAJpegImage));
                            //}
                            Ok(warp::reply::with_status(
                                warp::reply::html(format!("<h1>A file!</h1>")),
                                warp::http::StatusCode::OK,
                            ))
                        } else {
                            // Return an error if the field is not a file
                            Ok(warp::reply::with_status(
                                warp::reply::html(format!("<h1>Not a file</h1>")),
                                warp::http::StatusCode::BAD_REQUEST,
                            ))
                            //return Err(warp::reject::custom(UploadError::NotAFile));
                        }
                    })
                    .try_collect()
                    .await
                    .map_err(|e| {
                        // Return an error HTML response with the form processing error
                        warp::reply::with_status(
                            warp::reply::html(format!(
                                "<h1>Error processing form data: {}</h1>",
                                e
                            )),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    });
                // Return the HTML response
                //warp::reply::with_status(result.unwrap_or_else(|e| {
                //    // Return an error HTML response with the form processing error
                //    warp::reply::with_status(
                //        warp::reply::html(format!("<h1>Error processing form data</h1>")),
                //        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                //    )
                //}))
                //result.unwrap_or_else(|e| {
                //    // Return an error HTML response with the form processing error
                //    let mut v = Vec::new();
                //    v.push(warp::reply::with_status(
                //        warp::reply::html(format!("<h1>Error processing form data</h1>")),
                //        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                //    ));
                //    v
                //})
                result
            }
        })
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
