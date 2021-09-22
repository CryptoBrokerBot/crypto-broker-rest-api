use serde::Deserialize;
use tokio_postgres::{Config as PgConfig};

#[derive(Debug,Deserialize,Clone)]
pub struct DataSource {
  pub username : String,
  pub password : String,
  pub schema: String,
  pub host: String,
  pub port: u16
}

#[derive(Debug,Deserialize,Clone)]
pub struct Config {
  pub web_workers : u32,
  pub data_sources : Vec<DataSource>
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