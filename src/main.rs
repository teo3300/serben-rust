use hyper::header::{CACHE_CONTROL, CONTENT_TYPE};
// Import the environment module for argument parsing
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::{env, fs};
use std::path::Path;

mod server_env;
use server_env::Env;

static mut SERVE_ROOT: &str = ""; // Make SERVE_ROOT mutable and set it dynamically
static THUMBNAIL_EXTENSION: &str = "thumbnail";
static SOURCE_EXTENSION: &str = "source";

fn get_404(env: &Env) -> Result<Response<Body>, Infallible> {
    let filepath = format!("{}404.html", unsafe {SERVE_ROOT});
    let path = Path::new(&filepath);
    return if path.exists() {
        get_text_file(&filepath, env)
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
    println!("GET: [dir] {}", filepath);
    let path = Path::new(&filepath);
    let mut body = String::new();
    body.push_str("<!DOCTYPE html><head><style>
    .thumbnail-container {
        width: 200px;
        height: 200px;
        background-color: grey;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }
    .thumbnail-container img {
        max-height: 100%;
        max-width: 100%;
    }
</style></head><body>");
    body.push_str(&format!(
        "<h1>Index: /{}</h1>",
        path.to_str().unwrap().strip_prefix(unsafe { SERVE_ROOT }).unwrap()
    ));
    let parent = match path.parent() {
        Some(p) => match p.to_str().unwrap().strip_prefix(unsafe { SERVE_ROOT }) {
            Some(p) => p,
            None => "*",
        },
        None => Path::new("/").to_str().unwrap(),
    };
    body.push_str(&format!("<a href=\"/{}\">..</a><br>", parent));
    let mut entries = Vec::from_iter(fs::read_dir(&path).unwrap());
    entries.sort_by(
        |a, b|
        a.as_ref().unwrap().file_name().cmp(&b.as_ref().unwrap().file_name()));
    for entry in entries {
        let entry = entry.unwrap();
        let filename = entry.file_name();
        let filename = filename.to_str().unwrap();
        let path = entry.path();
        let path = path.to_str().unwrap().strip_prefix(unsafe { SERVE_ROOT }).unwrap();

        let case_ext;
        let ext = match filename.split('.').last() {
            Some(ext) => {
                case_ext = ext.to_lowercase();
                Some(case_ext.as_str())
            }
            None => None,
        };
        
        match ext {
            Some("jpg")
            | Some("png")
            | Some("gif")
            | Some("ico") => {
                body.push_str(&format!(
                    "<div class=\"thumbnail-container\">\
                        <img src=\"/{}.thumbnail\" alt=\"preview\">\
                    </div>\
                    <a href=\"/{}\">{}</a><br>",
                    path, path, filename
                ));
            }
            _ => {
                if !filename.starts_with('.') {
                    body.push_str(&format!("<a href=\"/{}\">{}</a><br>", path, filename));
                }
            }
        }
    }
    body.push_str("</body><p>- Source code available at -</p><footer><a href=\"https://github.com/teo3300/serben-rust\">serben-rust<a/><footer/></html>");
    return Ok(Response::new(Body::from(body)));
}

fn get_text_file(filepath: &str, env:&Env) -> Result<Response<Body>, Infallible> {
    let extension = Path::new(filepath).extension().unwrap_or_default().to_str().unwrap_or_default();
    let mime = env.get_mime(extension);

    println!("GET: [{:>3}] {}", extension, filepath);
    let path = Path::new(&filepath);
    return if path.exists() {
        match fs::read_to_string(&filepath) {
            Ok(content) => 
                Ok(Response::builder()
                    .header(CACHE_CONTROL, "public, max-age=600")
                    .header(CONTENT_TYPE, format!("text/{}", mime))
                    .body(Body::from(content))
                    .unwrap()),
            Err(err) => server_error(err),
        }
    } else {
        get_404(env)
    }
}

fn get_no_ext(filepath: &str, env: &Env) -> Result<Response<Body>, Infallible> {

    if Path::new(&filepath).is_dir() {
        return get_dir(&filepath);
    } else {
        // Useful having it returning the actual text file, for example with LICENSE etc.
        return get_text_file(&filepath, env);
    }

}

fn get_binary_file(filepath: &str, env: &Env) -> Result<Response<Body>, Infallible> {
    println!("GET: [bin] {}",filepath);

    return if Path::new(&filepath).exists() {
        match fs::read(&filepath) {
            Ok(content) => 
                Ok(Response::builder()
                    .header(CACHE_CONTROL, "public, max-age=3600")
                    .body(Body::from(content))
                    .unwrap()),
            Err(err) => server_error(err),
        }
    } else {
        get_404(env)
    }
}

use std::process::Command;

fn get_thumbnail(filepath: &str, env: &Env) -> Result<Response<Body>, Infallible> {
    // println!("? GET: [tmb] {}", filepath);
    let original_filepath = filepath.strip_suffix(&format!(".{}", THUMBNAIL_EXTENSION)).unwrap_or(filepath);

    if !Path::new(original_filepath).exists() {
        return get_404(env);
    }

    // Construct the thumbnail path
    let thumbnail_dir = format!("{}.cache/thumbnails/", unsafe { SERVE_ROOT });
    let thumbnail_path = format!("{}{}", thumbnail_dir, original_filepath.strip_prefix(unsafe { SERVE_ROOT }).unwrap().replace("/","_"));

    // Ensure the thumbnail directory exists
    if !Path::new(&thumbnail_dir).exists() {
        if let Err(err) = fs::create_dir_all(&thumbnail_dir) {
            return server_error(err);
        }
    }

    // Check if the thumbnail already exists
    if Path::new(&thumbnail_path).exists() {
        // Serve the generated thumbnail
        return get_binary_file(&thumbnail_path, env);
    }

    // Use ImageMagick to resize the image and save it to the thumbnail path
    let mut command = Command::new("convert");
    command.arg(original_filepath)
        .arg("-resize")
        .arg("10%")
        .arg("-resize")
        .arg("200x200<")    // never resize smaller than 200x200
        .arg("-resize")
        .arg("500x500>")
        .arg("-quality")
        .arg("20%")
        .arg(&thumbnail_path);

    if let Err(err) = command.status() {
        return server_error(err);
    }

    // Serve the generated thumbnail
    get_binary_file(&thumbnail_path, env)
}

async fn handle_request(req: Request<Body>, env: Env) -> Result<Response<Body>, Infallible> {
    let env = &env;
    let path = req.uri().path();

    return match path {
        "/" => get_no_ext(format!("{}index", unsafe { SERVE_ROOT }).as_str(), env),
        "/*" => get_dir(unsafe { SERVE_ROOT }),
        "/.cache" => get_404(env),                                  // Return 404 when listing cache
        path if path.starts_with("/.cache/") => get_404(env), // Return 404 for any path starting with "/thumbnails"
        _ => {
            let filename = &path["/".len()..];
            let filepath = format!("{}{}", unsafe { SERVE_ROOT }, filename);
            if let Some(extension) = Path::new(&filepath).extension() {
                match extension.to_str() {
                    Some("html")
                    | Some("css")
                    | Some("js")
                    | Some("txt")
                    | Some("md")
                    | Some("csv")
                    | Some("ics")
                    | Some("xml")
                    | Some("htm")
                    | Some("rss") => get_text_file(&filepath, env),
                    Some(ext) if ext == THUMBNAIL_EXTENSION => get_thumbnail(&filepath, env),
                    Some(ext) if ext == SOURCE_EXTENSION => get_text_file(
                        &filepath.strip_suffix(&format!(".{}", SOURCE_EXTENSION)).unwrap(), env),
                    _ => get_binary_file(&filepath, env),
                    // TODO: dang, I am really tempted to add arbitrary shell command execution here
                }
            } else {
                get_no_ext(&filepath, env)
            }
        },
    }
}

#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <serve_root>", args[0]);
        std::process::exit(1);
    }

    // Set SERVE_ROOT dynamically
    unsafe {
        SERVE_ROOT = Box::leak(args[1].clone().into_boxed_str());
    }

    let addr = ([0, 0, 0, 0], 8123).into();

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(|x| handle_request(x, Env::new())))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running on http://{}/* with SERVE_ROOT: {}", addr, unsafe { SERVE_ROOT });

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
