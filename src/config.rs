use serde::Deserialize;
use tokio_postgres::{Config as PgConfig};

#[derive(Debug,Deserialize,Clone)]
pub struct Config {
  pub data_source : DataSource
}

#[derive(Debug,Deserialize,Clone)]
pub struct DataSource {
  pub username : String,
  pub password : String,
  pub schema: String,
  pub host: String,
  pub port: u16
}

impl std::fmt::Display for DataSource {
  fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", "DataSource { ")?;
    write!(f, "username: {}, ", self.username)?;
    write!(f, "password: {}, ", self.password)?;
    write!(f, "schema: {}, ", self.schema)?;
    write!(f, "host: {}, ", self.host)?;
    write!(f, "port: {} ", self.port)?;
    write!(f, "{}", "}")?;
    Ok(())
  }
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

pub fn load_config() -> Config {
  return Config {
    data_source : DataSource {
      username : dotenv::var("CB_DBUSER").expect("Missing database username. Try adding `CB_DBUSER` environment variable."),
      password : dotenv::var("CB_DBPASS").expect("Missing database password. Try adding `CB_DBPASS` environment variable."),
      schema : dotenv::var("CB_DBSCHEMA").expect("Missing database schema. Try adding `CB_DBSCHEMA` environment variable."),
      port : dotenv::var("CB_DBPORT").ok().unwrap_or(String::from("5432")).parse::<u16>().expect("CB_DBPORT must be an unsigned integer 0-65535"),
      host : dotenv::var("CB_DBHOST").expect("Missing database host. Try adding `CB_DBHOST` environment variable.")
    }
  };
}