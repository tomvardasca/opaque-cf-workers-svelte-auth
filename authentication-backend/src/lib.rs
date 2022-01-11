mod utils;
mod confirmation_email;
mod data;
mod handlers;
mod opaque;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

use worker::{Date, Env, Request, Response, Result, Router, console_log, event, wasm_bindgen, wasm_bindgen_futures, worker_sys};

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log_request(&req);

    if req.method() == worker::Method::Options {
        if let Ok(mut response) = Response::ok("") {
            let headers = response.headers_mut();
            headers.set("Access-Control-Allow-Origin", "*")?;
            headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
            headers.set("Access-Control-Allow-Headers", "Content-Type")?;
            return Ok(response)
        }
        return Response::ok("")
    }

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", |_, _| Response::ok("Hello from Workers!"))
        .post_async("/register/start", handlers::register::start_handler)
        .post_async("/register/end", handlers::register::finish_handler)
        .get_async("/register/confirm/:username", handlers::register::confirm_mail_handler)
        .post_async("/login/start", handlers::login::start_handler)
        .post_async("/login/end", handlers::login::finish_handler)
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}
