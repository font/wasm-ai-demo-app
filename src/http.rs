use hyper::{Body, Method, Request, Response, StatusCode};
use tera::{Context, Tera};

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
pub async fn http_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut tera = Tera::default();
    match tera.add_raw_templates(vec![
        ("base.html", BASE_HTML_TEMPLATE),
        ("index.html", INDEX_HTML_TEMPLATE),
    ]) {
        Ok(_) => (),
        Err(_) => return internal_server_error_result(),
    };

    let model_data: &[u8] = include_bytes!("models/mobilenet.pt");

    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => {
            let mut context = Context::new();
            let output = match tera.render("index.html", &context) {
                Ok(output) => output,
                Err(_) => return internal_server_error_result(),
            };
            Ok(Response::new(Body::from(output)))
        }

        (&Method::POST, "/classify") => {
            let buf = hyper::body::to_bytes(req.into_body()).await?;
            //let flat_img = wasmedge_tensorflow_interface::load_jpg_image_to_rgb8(&buf, 224, 224);

            //Ok(Response::new(Body::from(format!("{} is detected with {}/255 confidence", class_name, max_value))))
            Ok(Response::new(Body::from(format!("..."))))
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
        <li><p>POST data to /classify such as: `curl http://localhost:8080/classify -X POST --data-binary '@grace_hopper.jpg'`</p></li>
        <li><p>Use the below form to upload an image</p></li>

        <br></br>
        <p>Click on the "Choose File" button to select a file and then click "Upload Image":</p>

        <form action="classify" method="post" enctype="multipart/form-data">
            <input type="file" name="uploadedFile">
            <input type="submit" value="Upload Image">
        </form>
    </ol>
{% endblock body %}
"#;
