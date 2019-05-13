use crate::validators::{create_issues, Response};
use actix_web::{get, web::Json, web::Query, App, HttpServer, Responder};
use std::env;

#[derive(Deserialize)]
struct Params {
    url: String,
    max_size: Option<usize>,
}

#[get("/validate")]
fn validate(params: Query<Params>) -> Json<Response> {
    let i = create_issues(&params.url, params.max_size.unwrap_or(1000));
    log::info!("Finished validation: {}", &params.url);
    Json(i)
}

#[get("/")]
fn index() -> impl Responder {
    r#"GTFS Validation tool (https://github.com/etalab/transport-validator-rust)
Use it with /validate?url=https.//.../gtfs.zip"#
}

pub fn run_server() {
    let port = env::var("PORT").unwrap_or_else(|_| "7878".to_string());
    let bind = env::var("BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", bind, port);

    HttpServer::new(|| App::new().service(validate).service(index))
        .bind(addr.clone())
        .expect(&format!("impossible to bind address {}", &addr))
        .run()
        .unwrap()
}
