use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::fs;
use std::path::Path;

static SERVE_ROOT: &str = "content/";

fn get_404() -> Result<Response<Body>, Infallible> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap())
}

fn get_dir(filepath: &str) -> Result<Response<Body>, Infallible> {
    let path = Path::new(&filepath);
    let mut body = String::new();
    body.push_str("<html><body>");
    body.push_str(&format!("<h1>Index: /{}</h1>", path.to_str().unwrap().strip_prefix(SERVE_ROOT).unwrap()));
    let parent = match path.parent() {
        Some(p) => match p.to_str().unwrap().strip_prefix(SERVE_ROOT) {
            Some(p) => p,
            None => "*",
        }
        None => Path::new("/").to_str().unwrap(),
    };
    body.push_str(&format!("<a href=\"/{}\">..</a><br>", parent));
    for entry in fs::read_dir(&path).unwrap() {
        let entry = entry.unwrap();
        let filename = entry.file_name();
        let filename = filename.to_str().unwrap();
        let path = entry.path();
        let path = path.to_str().unwrap().strip_prefix(SERVE_ROOT).unwrap();
        body.push_str(&format!("<a href=\"/{}\">{}</a><br>", path, filename));
    }
    body.push_str("</body></html>");
    return Ok(Response::new(Body::from(body)));
}

fn get_text_file(filepath: &str) -> Result<Response<Body>, Infallible> {
    match fs::read_to_string(filepath) {
        Ok(content) => Ok(Response::new(Body::from(content))),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Failed to read the file."))
            .unwrap()),
    }
}

fn get_dir_or_file(filepath: &str, ext: Option<&str>) -> Result<Response<Body>, Infallible> {
    let path = Path::new(&filepath);

    if path.is_dir() {
        return get_dir(&filepath);
    }
    
    let filepath = match ext {
        Some(_) => filepath.to_string(),
        None => format!("{}.html", &filepath),
    };
    let path = Path::new(&filepath);

    if path.exists() {
        get_text_file(&filepath)
    } else {
        get_404()
    }
}

fn get_binary_file(filepath: &str) -> Result<Response<Body>, Infallible> {
    if Path::new(&filepath).exists() {
        match fs::read(&filepath) {
            Ok(content) => Ok(Response::new(Body::from(content))),
            Err(_) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to read the file."))
                .unwrap()),
        }
    } else {
        get_404()
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();

    match path {
        "/" => get_dir_or_file(format!("{}index.html", SERVE_ROOT).as_str(), Some("html")),
        "/*" =>get_dir(SERVE_ROOT),
        _ => {
            let filename = &path["/".len()..];
            let filepath = format!("{}{}", SERVE_ROOT, filename);
            if let Some(extension) = Path::new(&filepath).extension() {
                match extension.to_str() {
                    Some("html")
                    | Some("txt")
                    | Some("css")
                    | Some("js")
                    | Some("xml")
                    | Some("rss") => get_text_file(&filepath),
                    _ => get_binary_file(&filepath),
                }
            } else {
                get_dir_or_file(&filepath, None)
            }
        },
    }
}

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}