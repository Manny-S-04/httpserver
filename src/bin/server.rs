use httpserver::http::{Request, Response, status_page, text};
use httpserver::server::{listen_on, serve};
use httpserver::syscalls::close;
use httpserver::*;

fn route(req: &Request) -> Response {
    let get = req.method == "GET" || req.method == "HEAD";
    match (get, req.path.as_str()) {
        (true, "/") => text(200, "hello from httpserver\n"),
        (true, _) => status_page(404),
        (false, _) => status_page(405),
    }
}

fn main() {
    let fd = match listen_on(INADDR_ANY, PORT) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("listen on {PORT}: {e:?}");
            std::process::exit(1);
        }
    };
    println!("listening on 0.0.0.0:{PORT}");

    let res = serve(fd, route, usize::MAX);
    let _ = close(fd);
    if let Err(e) = res {
        eprintln!("accept: {e:?}");
        std::process::exit(1);
    }
}
