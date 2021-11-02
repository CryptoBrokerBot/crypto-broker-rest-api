use crate::BrokerMapper;
use serde;
use serde::{Serialize,Deserialize};
use chrono::{DateTime,Utc};
#[allow(bare_trait_objects)]
pub type StdError = std::error::Error;
pub type StdResult<T> = Result<T,Box<dyn std::error::Error>>;
pub type Numeric = rust_decimal::Decimal;
use crate::datetime_formatters::datetime_Ymd_hms;

#[inline(always)]
pub fn new_std_err(msg : &str) -> Box<std::io::Error>{
    use std::io::{Error,ErrorKind};
    Box::new(Error::new(ErrorKind::Other,msg))
}

#[derive(Serialize,Clone,Debug)]
pub struct StatusResponse {
  pub success : bool,
  pub error_msg : Option<String>
}

impl StatusResponse {
  pub fn ok() -> StatusResponse {
    StatusResponse {
      success : true,
      error_msg : None
    }
  }
}

#[derive(Serialize,Clone,Debug)]
pub struct CurrencyData {
  // #[serde(rename = "asOf")]
  #[serde(with = "datetime_Ymd_hms", rename = "asOf")]
  pub as_of : DateTime<Utc>,
  pub id : String,
  pub symbol : String,
  pub name : String,
  pub price : Numeric,
  #[serde(rename="imageUrl")]
  pub image_url : String,
  #[serde(rename = "marketCap")]
  pub market_cap : Numeric,
  pub volume : Numeric,
  #[serde(rename = "coingeckoTimestamp")]
  pub coingecko_timestamp : String
}

#[derive(Serialize,Clone,Debug)]
pub struct CandleStickData {
  #[serde(with = "datetime_Ymd_hms", rename = "openDateTime")]
  pub open_date_time : DateTime<Utc>,
  pub open : Numeric,
  pub close : Numeric,
  pub low : Numeric,
  pub high : Numeric
}

#[derive(Serialize,Clone,Debug)]
pub struct TimeSeriesData {
  #[serde(with = "datetime_Ymd_hms", rename = "asOf")]
  pub as_of : DateTime<Utc>,
  pub price : Numeric
}

#[derive(Serialize,Clone)]
pub struct Position {
  pub name : String,
  pub crypto_id : String,
  // change serialize name
  #[serde(rename = "currentValue")]
  pub current_value : Numeric,
  pub qty : Numeric,
}

#[derive(Serialize,Clone)]
pub struct Portfolio {
  pub balance : Numeric,
  pub positions : Vec<Position>
}

pub struct RootAppState {
    pub broker_mapper : BrokerMapper
}