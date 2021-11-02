// For types specific to the API only

use serde::{Deserialize,Serialize};
use serde;
use crate::types::*;
use chrono::{DateTime,Utc};
use chrono::serde::ts_seconds_option;
use crate::datetime_formatters::*;

#[derive(Deserialize,Clone,Debug)]
pub struct GetWalletBalanceRequest {
  pub user_id : String
}

#[derive(Deserialize,Clone,Debug)]
pub struct DailyRewardRequest {
  pub user_id : String
}

#[derive(Deserialize,Clone,Debug)]
pub struct UpdateServerMembersRequest {
  pub server_id : String,
  pub user_ids : Vec<String>
}

#[derive(Deserialize,Clone,Debug)]
pub struct GetPortfolioRequest {
  pub user_id : String
}

#[derive(Deserialize,Clone,Debug)]
pub struct CoinIdentifierKey {
  pub crypto_id : Option<String>,
  pub name : Option<String>,
  pub symbol : Option<String>
}

#[derive(Deserialize,Clone,Debug)]
/// Describes a requested transaction. Each transaction has a coin key, user id, and a qty of coin to be bought or sold
pub struct CoinTransactionRequest {
  pub user_id : String,
  pub qty : Numeric,
  /// Allows the request to specify any of the fields in CoinIdentiferKey, and the server will try to resolve the correct coin from the info given if possible
  #[serde(flatten)]
  pub coin_key : CoinIdentifierKey
}

#[derive(Serialize,Debug)]
pub struct CoinTransactionResponse {
  pub msg : String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub currencies : Option<Vec<CurrencyData>>
}



#[derive(Deserialize,Debug,Clone)]
pub enum GraphGranularity {
  IntraDay,
  Daily,
  Weekly,
  Monthly,
  Quarterly,
  Anually
}


#[derive(Deserialize, Debug, Clone)]
pub struct GraphGenerationOptions {
  pub width : u32,
  pub height : u32,
  pub caption : String,
  pub from : DateTime<Utc>,
  pub to : DateTime<Utc>,
  pub granularity : GraphGranularity
}

#[derive(Deserialize,Debug,Clone)]
pub struct CoinPerformanceRequest {
  #[serde(flatten)]
  pub coin_key : CoinIdentifierKey,
  
  pub width : u32,
  pub height : u32,
  pub caption : Option<String>,
  #[serde(with = "ts_seconds_option")]
  pub from : Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option")]
  pub to : Option<DateTime<Utc>>,
  pub granularity : Option<GraphGranularity>
}