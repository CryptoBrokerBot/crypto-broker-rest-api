use std::io::BufReader;
use std::fs::File;

use actix_web::{get, web, App, HttpServer, Responder};
use serde_yaml;
use dotenv::dotenv;

use types::*;
use persistence::BrokerMapper;
use config::load_config;

mod config;
pub mod types;
pub mod persistence;
mod api;
mod middlewares;

// Validates API keys when in release build
#[cfg(not(debug_assertions))]
fn api_key_validatorer(api_keys : Vec<String>) -> impl FnMut(Option<&str>) -> bool {
    // Safe to use unwrap_or, since at the point contains is evaluated we've already determined 
    // we have a value. So "" should never happen, but we cannot just use unwrap() since that
    // will be evaluated before the condition and will cause a panic if there isn't an API key.
    move |s : Option<&str>| s.is_some() && api_keys.contains(&s.unwrap_or("").to_string())
}

// Ignores API key in debug build
#[cfg(debug_assertions)]
fn api_key_validatorer(_ : Vec<String>) -> impl FnMut(Option<&str>) -> bool {
    |_| true
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();
    let config = load_config();
    let broker_config = config.data_source.clone();
    let api_keys = BrokerMapper::new(&config.data_source).api_keys().await.expect("Unable to load API keys.");
    HttpServer::new(move || 
        App::new()
            .data(RootAppState{ broker_mapper: BrokerMapper::new(&broker_config) })
            .wrap(middlewares::apikey::ApiKeyService::from_validator(api_key_validatorer(api_keys.clone())))
            .wrap(middlewares::error::ErrorHandlerService)
            .service(api::routes::list)
            .service(api::routes::balance)
            .service(api::routes::daily_reward)
            .service(api::routes::update_server_members)
    )
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
