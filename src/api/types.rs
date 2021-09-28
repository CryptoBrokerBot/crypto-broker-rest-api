// For types specific to the API only

use serde::{Deserialize,Serialize};
use crate::types::*;

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
pub struct BuyCoinRequest {
  pub user_id : String,
  pub qty : Numeric,
  #[serde(flatten)]
  pub coin_key : CoinIdentifierKey
}

#[derive(Serialize,Debug)]
pub struct BuyCoinResponse {
  pub msg : String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub currencies : Option<Vec<CurrencyData>>
}