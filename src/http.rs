use lazy_static::lazy_static;
use regex::Regex;
use std::{fs, fs::File};
//use std::fs::File;
use std::io::Write;

use hyper::{body::Buf, header, header::HeaderValue, Body, Method, Request, Response, StatusCode};
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
            let mut file = match File::create(TEMP_FILENAME) {
                Ok(file) => file,
                Err(e) => {
                    println!("Error creating file {}: {}", TEMP_FILENAME, e);
                    return internal_server_error_result();
                }
            };
            match file.write_all(&buf) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error writing buffer to file: {}", e);
                    return internal_server_error_result();
                }
            };
            let mut context = Context::new();
            context.insert("path_to_image", TEMP_FILENAME);
            let output = match tera.render("classify.html", &context) {
                Ok(output) => output,
                Err(e) => {
                    println!("Error rendering classify.html: {}", e);
                    return internal_server_error_result();
                }
            };
            Ok(Response::new(Body::from(output)))
        }

        (&Method::POST, "/upload") => {
            let buf = hyper::body::to_bytes(req.into_body()).await?;
            let (parts, body) = req.into_parts();
            let form_data = match formdata::read_formdata(&mut buf.reader(), parts.headers) {
                Ok(data) => data,
                Err(e) => {
                    println!("Error reading form data: {}", e);
                    return internal_server_error_result();
                }
            };
            for (name, value) in form_data.fields {
                println!("Posted field name={} value={}", name, value);
            }
            for (name, file) in form_data.files {
                println!("Posted file name={} value={}", name, file.path);
            }
            // TODO(font): use unique file ID and to somehow link it to the client requesting this.
            let mut file = match File::create(TEMP_FILENAME) {
                Ok(file) => file,
                Err(e) => {
                    println!("Error creating file {}: {}", TEMP_FILENAME, e);
                    return internal_server_error_result();
                }
            };
            match file.write_all(&buf) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error writing buffer to file: {}", e);
                    return internal_server_error_result();
                }
            };
            let mut context = Context::new();
            context.insert("path_to_image", TEMP_FILENAME);
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
            let image_buf = match fs::read(TEMP_FILENAME) {
                Ok(buf) => buf,
                Err(e) => {
                    println!("Error reading {}: {}", TEMP_FILENAME, e);
                    return internal_server_error_result();
                }
            };
            //let content_type = match HeaderValue::from_static("image/jpg") {
            //    Ok(ct) => ct,
            //    Err(e) => {
            //        println!("Error creating HeaderValue from str \"image/jpg\": {}", e);
            //        return internal_server_error_result();
            //    }
            //};
            let mut response = Response::new(Body::from(image_buf));
            match response
                .headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"))
            {
                _ => (),
            };
            Ok(response)
            //let response = match Response::builder()
            //    .status(StatusCode::OK)
            //    .header(header::CONTENT_TYPE, "image/jpg")
            //    .body(image_buf)
            //{
            //    Ok(r) => r,
            //    Err(e) => {
            //        println!("Error creating response: {}", e);
            //        return internal_server_error_result();
            //    }
            //};
            //Ok(Response::new(response))
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

const TEMP_FILENAME: &str = "temp.jpg";

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
{% endblock body %}
"#;
