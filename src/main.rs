use std::io::BufReader;
use std::fs::File;

use actix_web::{get, web, App, HttpServer, Responder};
use serde_yaml;

use types::*;
use persistence::BrokerMapper;
use config::Config;

mod config;
pub mod types;
pub mod persistence;
mod api;
mod middlewares;

fn load_config() -> serde_yaml::Result<Config> {
    let cfg_file = File::open("config.yaml").expect("Could not find 'config.yaml'.");
    serde_yaml::from_reader(BufReader::new(cfg_file))
}

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
    let config = load_config().expect("ahhh");
    let broker_conf  = config.data_sources.get(0).expect("No data sources specified in config file!").clone();
    let api_keys = BrokerMapper::new(&broker_conf.clone()).api_keys().await.expect("Unable to load API keys.");
    HttpServer::new(move || 
        App::new()
            .data(RootAppState{ broker_mapper: BrokerMapper::new(&broker_conf) })
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
