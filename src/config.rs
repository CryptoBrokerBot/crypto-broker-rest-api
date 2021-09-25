use tokio_postgres::{Config as PgConfig};
use crate::types::*;

#[derive(Debug,Clone)]
pub struct DataSource {
  pub username : String,
  pub password : String,
  pub schema: String,
  pub host: String,
  pub port: u16
}

#[derive(Debug,Clone)]
pub struct Config {
  pub data_source : DataSource
}

impl From<&DataSource> for PgConfig {
  fn from(ds : &DataSource) -> PgConfig {
    let mut cfg = PgConfig::new();
    cfg.user(ds.username.as_str())
       .password(ds.password.as_str())
       .dbname(ds.schema.as_str())
       .host(ds.host.as_str())
       .port(ds.port);
    return cfg;
  }
}

pub fn load_config() -> StdResult<Config> {
  use std::env::var;
  let username = var("CB_DBUSER")?;
  let password = var("CB_DBPASS")?;
  let schema = var("CB_DBNAME")?;
  let host = var("CB_DBHOST")?;
  let port : u16 = var("CP_DBPORT").map_or(5432,|v| v.parse::<u16>().unwrap());
  Ok(Config{data_source: DataSource{username,password,schema,host,port}})

}