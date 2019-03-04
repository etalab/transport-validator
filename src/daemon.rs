use crate::validators::validate;
use gotham::helpers::http::response::create_response;
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

fn router() -> Router {
    build_simple_router(|route| {
        route
            .get("/validate")
            .with_query_string_extractor::<QueryStringExtractor>()
            .to(validation_handler);
    })
}

pub fn run_server() {
    let port = env::var("PORT").unwrap_or_else(|_| "7878".to_string());
    let bind = env::var("BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", bind, port);
    gotham::start(addr, router())
}
