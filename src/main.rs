use std::env; // Import the environment module for argument parsing
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::io::Read;
use std::process::Stdio;
use std::convert::Infallible;
use std::fs;
use std::path::Path;

static mut SERVE_ROOT: &str = ""; // Make SERVE_ROOT mutable and set it dynamically
static THUMBNAIL_EXTENSION: &str = "thumbnail";
static SOURCE_EXTENSION: &str = "source";

fn get_404() -> Result<Response<Body>, Infallible> {
    let filepath = format!("{}404.html", unsafe {SERVE_ROOT});
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
    println!("GET: [dir] {}", filepath);
    let path = Path::new(&filepath);
    let mut body = String::new();
    body.push_str("<html><head><style>
    .thumbnail-container {
        width: 100px;
        height: 100px;
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
            | Some("tiff") => {
                body.push_str(&format!(
                    "<div class=\"thumbnail-container\">\
                        <img src=\"/{}.thumbnail\" alt=\"preview\">\
                    </div>\
                    <a href=\"/{}\">{}</a><br>",
                    path, path, filename
                ));
            }
            _ => {
                body.push_str(&format!("<a href=\"/{}\">{}</a><br>", path, filename));
            }
        }
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
            Err(err) => server_error(err),
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
        .arg("10%")
        .arg("-resize")
        .arg("200x200<")    // never resize smaller than 512x512
        .arg("-resize")
        .arg("500x500>")
        .arg("-quality")
        .arg("20%")
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
        "/" => get_no_ext(format!("{}index", unsafe { SERVE_ROOT }).as_str()),
        "/*" =>get_dir(unsafe { SERVE_ROOT }),
        _ => {
            let filename = &path["/".len()..];
            let filepath = format!("{}{}", unsafe { SERVE_ROOT }, filename);
            if let Some(extension) = Path::new(&filepath).extension() {
                match extension.to_str() {
                    Some("html")
                    | Some("txt")
                    | Some("css")
                    | Some("js")
                    | Some("xml")
                    | Some("rss") => get_text_file(&filepath),
                    Some(ext) if ext == THUMBNAIL_EXTENSION => get_thumbnail(&filepath),
                    Some(ext) if ext == SOURCE_EXTENSION => get_text_file(
                        &filepath.strip_suffix(&format!(".{}", SOURCE_EXTENSION)).unwrap()),
                    _ => get_binary_file(&filepath),
                    // TODO: dang, I am really tempted to add arbitrary shell command execution here
                }
            } else {
                get_no_ext(&filepath)
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
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running on http://{} with SERVE_ROOT: {}", addr, unsafe { SERVE_ROOT });

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
