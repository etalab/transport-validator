use crate::custom_rules;
use crate::validate::{generate_validation_from_reader, process, Response};
use actix_web::{get, post, web, web::Json, App, Error, HttpServer};
use futures::StreamExt;
use serde::Deserialize;
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
async fn validate(params: web::Query<Params>) -> Result<Json<Response>, Error> {
    log::info!("Starting validation: {}", &params.url);
    let gtfs = gtfs_structures::RawGtfs::from_url_async(&params.url).await;

    let custom_rules = custom_rules::CustomRules {
        ..Default::default()
    };
    let result = process(gtfs, params.max_size.unwrap_or(1000), &custom_rules);
    log::info!("Finished validation");
    Ok(Json(result))
}

#[get("/")]
async fn index() -> &'static str {
    r#"GTFS Validation tool (https://github.com/etalab/transport-validator-rust)
Use it with /validate?url=https://.../gtfs.zip

This software is open-source.
See the code and the documentation: https://github.com/etalab/transport-validator"#
}

#[post("/validate")]
async fn validate_post(
    params: web::Query<PostParams>,
    mut payload: web::Payload,
) -> Result<Json<Response>, Error> {
    let max_size = params.max_size.unwrap_or(1000);

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }
    let reader = std::io::Cursor::new(body);
    let custom_rules = custom_rules::CustomRules {
        ..Default::default()
    };

    Ok(Json(generate_validation_from_reader(
        reader,
        max_size,
        &custom_rules,
    )))
}

pub fn run_server() -> std::io::Result<()> {
    run_server_impl()
}

#[actix_rt::main]
async fn run_server_impl() -> std::io::Result<()> {
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
    .unwrap_or_else(|_| panic!("impossible to bind address {}", &addr))
    .run()
    .await
}
