use hyper::{Body, Method, Request, Response, StatusCode};

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
pub async fn http_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let model_data: &[u8] = include_bytes!("models/mobilenet.pt");

    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(Body::from(HTML_CODE))),

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

const HTML_CODE: &'static str = r#"
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
    <h1>To classify an image try one of the options below:</h1>
    <ol>
        <li><p>POST data to /classify such as: `curl http://localhost:8080/classify -X POST --data-binary '@grace_hopper.jpg'`</p></li>
        <li><p>Use the below form to upload an image</p></li>
    </ol>
    
</body>
</html>
"#;
