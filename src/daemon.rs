use crate::validators::validate;
use gotham::helpers::http::response::create_response;
use gotham::middleware::logger::SimpleLogger;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::{builder::*, Router};
use gotham::state::{FromState, State};
use hyper::{Body, Response, StatusCode};
use std::env;

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct QueryStringExtractor {
    url: String,
    max_size: Option<usize>,
}

fn validation_handler(mut state: State) -> (State, Response<Body>) {
    let query_param = QueryStringExtractor::take_from(&mut state);

    let res = match validate(&query_param.url, query_param.max_size.unwrap_or(1000)) {
        Ok(json) => create_response(
            &state,
            StatusCode::OK,
            mime::APPLICATION_JSON,
            json.into_bytes(),
        ),
        Err(err) => create_response(
            &state,
            StatusCode::INTERNAL_SERVER_ERROR,
            mime::APPLICATION_JSON,
            format!("{{\"error\": \"{}\"}}", err).into_bytes(),
        ),
    };
    log::info!("Finnished validation: {}", &query_param.url);
    (state, res)
}

fn index(state: State) -> (State, &'static str) {
    (
        state,
        "GTFS Validation tool (https://github.com/etalab/transport-validator-rust)\n
Use it with /validation?url=https.//.../gtfs.zip",
    )
}

fn router() -> Router {
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(SimpleLogger::new(log::Level::Info))
            .build(),
    );
    build_router(chain, pipelines, |route| {
        route
            .get("/validate")
            .with_query_string_extractor::<QueryStringExtractor>()
            .to(validation_handler);

        route.get("/").to(index);
    })
}

pub fn run_server() {
    let port = env::var("PORT").unwrap_or_else(|_| "7878".to_string());
    let bind = env::var("BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", bind, port);
    gotham::start(addr, router())
}
