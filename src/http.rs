use lazy_static::lazy_static;
use regex::Regex;
use std::io::Write;
use std::{fs, fs::File};

use hyper::{header, header::HeaderValue, Body, Method, Request, Response, StatusCode};
use tera::{Context, Tera};

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
pub async fn http_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut tera = Tera::default();
    match tera.add_raw_templates(vec![
        ("base.html", BASE_HTML_TEMPLATE),
        ("index.html", INDEX_HTML_TEMPLATE),
        ("classify.html", CLASSIFY_HTML_TEMPLATE),
    ]) {
        Ok(_) => (),
        Err(_) => return internal_server_error_result(),
    };

    println!("{} {}", req.method(), req.uri().path());
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => {
            let context = Context::new();
            let output = match tera.render("index.html", &context) {
                Ok(output) => output,
                Err(e) => {
                    println!("Error rendering index.html: {}", e);
                    return internal_server_error_result();
                }
            };
            Ok(Response::new(Body::from(output)))
        }

        (&Method::POST, "/classify") => {
            let buf = hyper::body::to_bytes(req.into_body()).await?;
            // TODO(font): use unique file ID and to somehow link it to the client requesting this.
            let mut image_file = match File::create(TEMP_IMAGE_NAME) {
                Ok(image_file) => image_file,
                Err(e) => {
                    println!("Error creating file {}: {}", TEMP_IMAGE_NAME, e);
                    return internal_server_error_result();
                }
            };
            match image_file.write_all(&buf) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error writing buffer to file: {}", e);
                    return internal_server_error_result();
                }
            };

            let results = crate::inference::infer_image(TEMP_IMAGE_NAME);
            let mut context = Context::new();
            context.insert("path_to_image", TEMP_IMAGE_NAME);
            context.insert("inference_result", &results);
            let output = match tera.render("classify.html", &context) {
                Ok(output) => output,
                Err(e) => {
                    println!("Error rendering classify.html: {}", e);
                    return internal_server_error_result();
                }
            };
            Ok(Response::new(Body::from(output)))
        }

        _ if IMAGES_PATH.is_match(req.uri().path()) => {
            let image_buf = match fs::read(TEMP_IMAGE_NAME) {
                Ok(buf) => buf,
                Err(e) => {
                    println!("Error reading {}: {}", TEMP_IMAGE_NAME, e);
                    return internal_server_error_result();
                }
            };
            let mut response = Response::new(Body::from(image_buf));
            match response
                .headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"))
            {
                _ => (),
            };
            Ok(response)
        }
        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn internal_server_error_result() -> Result<Response<Body>, hyper::Error> {
    let mut server_error = Response::default();
    *server_error.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    Ok(server_error)
}

const TEMP_IMAGE_NAME: &str = "temp.jpg";

lazy_static! {
    static ref IMAGES_PATH: Regex = Regex::new("^/.*\\.jpg$").unwrap();
}

const BASE_HTML_TEMPLATE: &'static str = r#"
<!doctype html>

<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">

  <title>WASM AI Inferencing</title>
  <meta name="description" content="A WASM AI inferencing demo application.">
  <meta name="author" content="Ivan Font">
</head>

<body>
    {% block body %}{% endblock body %}
</body>
</html>
"#;

const INDEX_HTML_TEMPLATE: &'static str = r#"
{% extends "base.html" %}
{% block body %}
    <h1>To classify an image try one of the options below:</h1>
    <ol>
        <li><p>POST data to /classify such as: `curl http://localhost:8080/classify -X POST --data-binary '@image.jpg'`</p></li>
        <li><p>Use the below form to upload an image:</p></li>

        <p>Click on the "Choose File" button to select a file and then click "Upload Image":</p>

        <form action="upload" method="post" enctype="multipart/form-data">
            <input type="file" name="uploadedFile" accept=".jpg">
            <input type="submit" value="Upload Image">
        </form>
    </ol>
{% endblock body %}
"#;

const CLASSIFY_HTML_TEMPLATE: &'static str = r#"
{% extends "base.html" %}
{% block body %}
    <form action="/" method="get">
        <input type="button" value="back" onclick="history.back()">
    </form>
    <h1>Image to Infer:</h1>
    <img src="{{ path_to_image }}" alt="Inferencing image">
    <h2>Result:</h2>
    <p>{{ inference_result }}</p>
{% endblock body %}
"#;
