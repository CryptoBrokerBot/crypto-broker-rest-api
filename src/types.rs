use crate::BrokerMapper;
use serde;
use serde::{Serialize};
use chrono::{DateTime,Utc};
pub type StdError = std::error::Error;
pub type StdResult<T> = Result<T,Box<dyn std::error::Error>>;
pub type Numeric = rust_decimal::Decimal;

#[derive(Serialize,Clone,Debug)]
pub struct CurrencyData {
  // #[serde(rename = "asOf")]
  #[serde(with = "date_formatter", rename = "asOf")]
  pub as_of : DateTime<Utc>,
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

#[derive(Serialize,Clone)]
pub struct Position {
  pub name : String,
  pub symbol : String,
  // change serialize name
  #[serde(rename = "totalCost")]
  pub total_cost : f64,
  pub qty : f64,
}

#[derive(Serialize,Clone)]
pub struct Portfolio {
  pub balance : f64,
  pub positions : Vec<Position>
}


pub struct RootAppState {
    pub broker_mapper : BrokerMapper
}

// Copied from serde example https://serde.rs/custom-date-format.html
mod date_formatter {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    #[allow(dead_code)]
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}