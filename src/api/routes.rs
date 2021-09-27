use actix_web::{get, Responder, HttpResponse, web, post, put};

use crate::types::{*};
use super::types::{*};

#[get("/list")]
pub async fn list(state : web::Data<RootAppState>) -> StdResult<impl Responder> {
  Ok(HttpResponse::Ok().json(state.broker_mapper.list_currencies().await?))
}

#[get("/balance")]
pub async fn balance(state : web::Data<RootAppState>, params : web::Query<GetWalletBalanceRequest>) -> StdResult<impl Responder> {
  // TODO: Have a way to indicate the difference between a non-existant wallet and an actual error.
  Ok(HttpResponse::Ok().json(state.broker_mapper.get_wallet_balance_by_userid(&params.user_id).await?))
}

#[post("/daily-reward")]
pub async fn daily_reward(state : web::Data<RootAppState>, request : web::Query<DailyRewardRequest>) -> StdResult<impl Responder> {
  // TODO: In a single query only allow the user to increase his balance once daily.
  let curr_balance = state.broker_mapper.get_wallet_balance_by_userid(&request.user_id).await.unwrap_or(0.into());
  let new_balance = curr_balance + &100_i32.into();
  state.broker_mapper.set_wallet_balance_by_userid(&request.user_id, new_balance).await?;
  Ok(HttpResponse::Ok().json(StatusResponse::ok()))
}

#[put("leaderboard")]
pub async fn update_server_members(state : web::Data<RootAppState>, request : web::Query<UpdateServerMembersRequest>) -> StdResult<impl Responder> {
  state.broker_mapper.update_server_patrons(&request.user_ids, &request.server_id).await?;
  Ok(HttpResponse::Ok().json(StatusResponse::ok()))
}