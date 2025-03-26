use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::io::Read;
use std::process::Stdio;
use std::convert::Infallible;
use std::fs;
use std::path::Path;

static SERVE_ROOT: &str = "content/";
static THUMBNAIL_EXTENSION: &str = "thumbnail";

fn get_404() -> Result<Response<Body>, Infallible> {
    let filepath = format!("{}404.html", SERVE_ROOT);
    let path = Path::new(&filepath);
    return if path.exists() {
        get_text_file(&filepath)
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found."))
            .unwrap())
    }
}

fn server_error <T: std::fmt::Debug>(err: T) -> Result<Response<Body>, Infallible>  {
    let ret_val = Ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("Internal server error."))
        .unwrap());
    println!("Error: {:?}", err);
    return ret_val;
}

fn get_dir(filepath: &str) -> Result<Response<Body>, Infallible> {
    println!("GET: [dir] {}",filepath);
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
    println!("GET: [txt] {}",filepath);
    let path = Path::new(&filepath);
    return if path.exists() {
        match fs::read_to_string(&filepath) {
            Ok(content) => Ok(Response::new(Body::from(content))),
            Err(err) => server_error(err),
        }
    } else {
        get_404()
    }
}

fn get_no_ext(filepath: &str) -> Result<Response<Body>, Infallible> {

    if Path::new(&filepath).is_dir() {
        return get_dir(&filepath);
    } else {
        return get_text_file(&format!("{}.html", &filepath));
    }

}

fn get_binary_file(filepath: &str) -> Result<Response<Body>, Infallible> {
    println!("GET: [bin] {}",filepath);

    return if Path::new(&filepath).exists() {
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

use std::process::Command;

fn get_thumbnail(filepath: &str) -> Result<Response<Body>, Infallible> {
    println!("GET: [tmb] {}",filepath);
    let original_filepath = filepath.strip_suffix(&format!(".{}",THUMBNAIL_EXTENSION)).unwrap_or(filepath);

    if !Path::new(original_filepath).exists() {
        return get_404();
    }

    // Use ImageMagick to resize the image
    let mut command = Command::new("convert");
    command.arg(original_filepath)
        .arg("-resize")
        .arg("30%")
        .arg("-quality")
        .arg("80%")
        .arg("jpeg:-");

    let mut processed  = match command.stdout(Stdio::piped()).spawn()
    {
        Ok(process) => process,
        Err(err) => {
            println!("Running {:?}", command);
            return server_error(err)
        }
    };

    // Read the output from the ImageMagick process
    let mut output = Vec::new();
    if let Some(mut stdout) = processed.stdout.take() {
        if let Err(err) = stdout.read_to_end(&mut output) {
            return server_error(err);
        }
    }

    if let Err(err) = processed.wait() {
        return server_error(err);
    }

    return Ok(Response::new(Body::from(output)))

}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();

    return match path {
        "/" => get_no_ext(format!("{}index", SERVE_ROOT).as_str()),
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
                    Some(ext) if ext == THUMBNAIL_EXTENSION => get_thumbnail(&filepath),
                    _ => get_binary_file(&filepath),
                }
            } else {
                get_no_ext(&filepath)
            }
        },
    }
}

#[tokio::main]
async fn main() {
    let addr = ([0, 0, 0, 0], 8123).into();

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
