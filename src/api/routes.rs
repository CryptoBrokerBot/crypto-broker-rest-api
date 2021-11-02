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
pub async fn buy_currency(state : web::Data<RootAppState>, params : web::Json<CoinTransactionRequest>) -> StdResult<impl Responder> {
  let coin = match coin_from_key(&state, &params.coin_key).await {
    Ok(c) => c, Err(resp) => return Ok(resp)
  };
  state.broker_mapper.buy_currency(&coin.id,&params.qty,&params.user_id).await?;
  Ok((web::Json(CoinTransactionResponse{msg:String::from("Success"),currencies:None}),actix_web::http::StatusCode::OK))
}

#[post("/sell")]
pub async fn sell_currency(state : web::Data<RootAppState>, params : web::Json<CoinTransactionRequest>) -> StdResult<impl Responder> {
  let coin = match coin_from_key(&state, &params.coin_key).await {
    Ok(c) => c, Err(resp) => return Ok(resp)
  };
  state.broker_mapper.sell_currency(&coin.id, &params.qty, &params.user_id).await?;
  Ok((web::Json(CoinTransactionResponse{msg:String::from("Success"), currencies: None}),actix_web::http::StatusCode::OK))
}

#[inline(always)]
/// Resolves a coin identifer tuple to either a response (in the case that the identifier is ambiguous or no coin found), or a coin if exactly one could be found with the 
/// information present
async fn coin_from_key(state : &web::Data<RootAppState>, coin_key : &CoinIdentifierKey) -> Result<CurrencyData, (web::Json<CoinTransactionResponse>,actix_web::http::StatusCode)>{
  let coins_res : StdResult<Vec<CurrencyData>> = state.broker_mapper.get_coins_matching_key(coin_key).await;
  match coins_res {
    Ok(mut coins) => {
      if coins.is_empty() {
        return Err((web::Json(CoinTransactionResponse{msg:String::from("No coin found matching criteria!"),currencies:None}),actix_web::http::StatusCode::BAD_REQUEST));
      }
      if coins.len() > 1 {
        return Err((web::Json(CoinTransactionResponse{msg:String::from("Multiple coins found!"),currencies: Some(coins)}),actix_web::http::StatusCode::MULTIPLE_CHOICES));
      }
      return Ok(coins.remove(0));
    },
    Err(_) => {
      return Err((web::Json(CoinTransactionResponse{msg:String::from(format!("Failed retrieving coins for identifer tuple {:?}",coin_key)),currencies:None}),actix_web::http::StatusCode::INTERNAL_SERVER_ERROR));
    }
  }
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

#[get("/performance")]
pub async fn get_performance(state : web::Data<RootAppState>, params : web::Query<CoinPerformanceRequest>) -> actix_web::Result<impl Responder> {
  use chrono::{Utc};
  let coin = match coin_from_key(&state, &params.coin_key).await {
    Ok(c) => c, Err(resp) => {
      return Ok(actix_web::HttpResponseBuilder::new(resp.1).json(resp.0));
    }
  };

  let opts = GraphGenerationOptions {
    width : params.width,
    height : params.height,
    caption : params.caption.clone().unwrap_or(coin.id.clone()),
    from : params.from.clone().expect("Could not get from"),
    to : params.to.clone().expect("Could not get to"),
    granularity : params.granularity.clone().unwrap_or(GraphGranularity::IntraDay)
  };
  let time_series : Vec<CandleStickData> = state.broker_mapper.get_candle_sticks(&opts.from, &opts.to,&coin.id,&opts.granularity).await?;
  use crate::perf::CoinPerformanceGraph;
  let graph = CoinPerformanceGraph::from_candle_sticks(&time_series, &opts );
  
  Ok(graph.into())
}