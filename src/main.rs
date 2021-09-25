extern crate tokio;
mod config;
mod types;
mod persistence;
use types::*;
use config::*;
use persistence::CryptoPortfolioDao;
use actix_web::{get, web, App, HttpServer};




#[get("test")]
async fn get_test(app_state : web::Data<RootAppState>) -> StdResult<web::Json<Portfolio>> {
    let root_state = app_state.as_ref();
    let broker_mapper = &root_state.broker_mapper;
    // let currencies = broker_mapper.list_currencies().await?;
    // let balance = broker_mapper.get_wallet_balance(&"andrew").await?;
    let portfolio = broker_mapper.get_portfolio(&"andrew").await?;
    // use rust_decimal::prelude::*;
    // let _ = broker_mapper.create_wallet("andrewtest","andrewservertest",Numeric::from_f64(4000.0).expect("Could not convert f64 to numeric")).await?;
    Ok(web::Json(portfolio))
}



#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let config = load_config().expect("Could not load config from environment!");
    let broker_conf  = config.data_source.clone();
    HttpServer::new(move || App::new().service(get_test).data(RootAppState{ broker_mapper: CryptoPortfolioDao::new(&broker_conf)}))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}