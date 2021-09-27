// For types specific to the API only

use serde::{Deserialize};

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
