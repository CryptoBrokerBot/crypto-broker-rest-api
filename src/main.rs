extern crate tokio;
mod config;
mod types;
mod persistence;
use types::*;
use persistence::BrokerMapper;
use config::Config;
use std::io::BufReader;
use serde_yaml;
use std::fs::File;
use actix_web::{get, web, App, HttpServer,HttpResponse};

 

#[get("test")]
async fn get_test(app_state : web::Data<RootAppState>) -> StdResult<web::Json<Numeric>> {
    let root_state = app_state.as_ref();
    let broker_mapper = &root_state.broker_mapper;
    let balance = broker_mapper.get_wallet_balance("andrew").await?;
    Ok(web::Json(balance))
}

fn load_config() -> serde_yaml::Result<Config> {
    let cfg_file = File::open("config.yaml").expect("Could not find 'config.yaml'.");
    serde_yaml::from_reader(BufReader::new(cfg_file))
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let config = load_config().expect("ahhh");
    let broker_conf  = config.data_sources.get(0).expect("No data sources specified in config file!").clone();
    HttpServer::new(move || App::new().service(get_test).data(RootAppState{ broker_mapper: BrokerMapper::new(&broker_conf)}))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}