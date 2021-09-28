use actix_web::{get, Responder, HttpResponse, web, post, put};
use crate::types::{*};
use super::types::{*};

macro_rules! json_ok {
  ($e : expr) => {
    Ok(HttpResponse::Ok().json($e))
  };
}

#[get("/list")]
pub async fn list(state : web::Data<RootAppState>) -> StdResult<impl Responder> {
  json_ok!(state.broker_mapper.list_currencies().await?)
}

#[get("/balance")]
pub async fn balance(state : web::Data<RootAppState>, params : web::Query<GetWalletBalanceRequest>) -> StdResult<impl Responder> {
  // TODO: Have a way to indicate the difference between a non-existant wallet and an actual error.
  json_ok!(state.broker_mapper.get_wallet_balance_by_userid(&params.user_id).await?)
}


#[get("/portfolio")]
pub async fn get_portfolio(state : web::Data<RootAppState>, params : web::Query<GetPortfolioRequest>) -> StdResult<impl Responder> {
  json_ok!(state.broker_mapper.get_portfolio(&params.user_id).await?)
}



#[post("/buy")]
pub async fn buy_currency(state : web::Data<RootAppState>, params : web::Query<BuyCoinRequest>) -> StdResult<impl Responder> {
  let coins : Vec<CurrencyData> = state.broker_mapper.get_coins_matching_key(&params.coin_key).await?;
  if coins.is_empty() {
    return Ok((web::Json(BuyCoinResponse{msg:String::from("No coin found matching criteria!"),currencies:None}),actix_web::http::StatusCode::BAD_REQUEST));
  }
  if coins.len() > 1 {
    return Ok((web::Json(BuyCoinResponse{msg:String::from("Multiple coins found!"),currencies: Some(coins)}),actix_web::http::StatusCode::BAD_REQUEST));
  }
  let coin_id : &CurrencyData = coins.get(0).unwrap();

  state.broker_mapper.buy_currency(&coin_id.id,&params.qty,&params.user_id).await?;
  Ok((web::Json(BuyCoinResponse{msg:String::from("Success"),currencies:None}),actix_web::http::StatusCode::OK))
}


#[get("/coin")]
pub async fn get_coin(state : web::Data<RootAppState>, params : web::Query<CoinIdentifierKey>) -> StdResult<impl Responder> {
  return json_ok!(state.broker_mapper.get_coins_matching_key(&params).await?);
}


#[post("/daily-reward")]
pub async fn daily_reward(state : web::Data<RootAppState>, request : web::Query<DailyRewardRequest>) -> StdResult<impl Responder> {
  // TODO: In a single query only allow the user to increase his balance once daily.
  let curr_balance = state.broker_mapper.get_wallet_balance_by_userid(&request.user_id).await.unwrap_or(0.into());
  let new_balance = curr_balance + &100_i32.into();
  state.broker_mapper.set_wallet_balance_by_userid(&request.user_id, new_balance).await?;
  json_ok!(StatusResponse::ok())
}

#[put("leaderboard")]
pub async fn update_server_members(state : web::Data<RootAppState>, request : web::Query<UpdateServerMembersRequest>) -> StdResult<impl Responder> {
  state.broker_mapper.update_server_patrons(&request.user_ids, &request.server_id).await?;
  json_ok!(StatusResponse::ok())
}