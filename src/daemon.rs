use crate::validators::{create_issues_from_reader, process, Response};
use actix_web::{get, post, web, web::Json, App, Error, HttpServer, Responder};
use futures::{future::ok, Future, Stream};
use std::env;

#[derive(Deserialize)]
struct Params {
    url: String,
    max_size: Option<usize>,
}

#[derive(Deserialize)]
struct PostParams {
    max_size: Option<usize>,
}

#[get("/validate")]
fn validate(params: web::Query<Params>) -> impl Future<Item = Json<Response>, Error = Error> {
    log::info!("Starting validation: {}", &params.url);
    gtfs_structures::RawGtfs::from_url_async(&params.url)
        .from_err()
        .and_then(move |gtfs| {
            let result = process(Ok(gtfs), params.max_size.unwrap_or(1000));
            log::info!("Finished validation");
            ok(Json(result))
        })
}

#[get("/")]
fn index() -> impl Responder {
    r#"GTFS Validation tool (https://github.com/etalab/transport-validator-rust)
Use it with /validate?url=https://.../gtfs.zip"#
}

#[post("/validate")]
fn validate_post(
    params: web::Query<PostParams>,
    body: web::Payload,
) -> impl Future<Item = Json<Response>, Error = Error> {
    let max_size = params.max_size.unwrap_or(1000);
    body.map_err(Error::from)
        .fold(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            Ok::<_, Error>(body)
        })
        .and_then(move |body| {
            let reader = std::io::Cursor::new(body);
            Ok(Json(create_issues_from_reader(reader, max_size)))
        })
}

pub fn run_server() {
    let port = env::var("PORT").unwrap_or_else(|_| "7878".to_string());
    let bind = env::var("BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", bind, port);

    HttpServer::new(|| {
        App::new()
            .service(validate)
            .service(index)
            .service(validate_post)
    })
    .bind(addr.clone())
    .expect(&format!("impossible to bind address {}", &addr))
    .run()
    .unwrap()
}
