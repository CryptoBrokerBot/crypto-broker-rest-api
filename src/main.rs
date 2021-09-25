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

fn load_config() -> serde_yaml::Result<Config> {
    let cfg_file = File::open("config.yaml").expect("Could not find 'config.yaml'.");
    serde_yaml::from_reader(BufReader::new(cfg_file))
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let config = load_config().expect("ahhh");
    let broker_conf  = config.data_sources.get(0).expect("No data sources specified in config file!").clone();
    HttpServer::new(move || 
        App::new()
            .data(RootAppState{ broker_mapper: BrokerMapper::new(&broker_conf) })
            .service(api::routes::list)
            .service(api::routes::balance)
            .service(api::routes::daily_reward)
            .service(api::routes::update_server_members)
    )
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
