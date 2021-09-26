use tokio_postgres::{Config as PgConfig,Transaction,Row,NoTls,Client, types::*};
use crate::config::DataSource;
use crate::types::*;
use std::convert::TryFrom;


#[derive(Debug)]
pub struct BrokerMapper {
  config : PgConfig,
}


macro_rules! get_client {
  ($a : ident) => {
    {
      let (client,conn) = $a.connect(NoTls).await?;
      tokio::spawn(async move{
        if let Err(e) = conn.await {
          eprintln!("connection error: {}",e);
        }
      });
      client
    }
  };
}

async fn get_wallet_balance_by_userid(user_id : &str, client : &Client) -> StdResult<Numeric> {
  let query = r#"
  SELECT walletBalance FROM wallets WHERE userId = $1 LIMIT 1;
  "#;
  let amount : Numeric = client.query_one(query, &[&user_id]).await?.try_get("walletBalance")?;
  return Ok(amount);
}

async fn update_server_patrons(user_ids : &Vec<String>, server_id : &str, client : &Client) -> StdResult<()> {
  // TODO: Consider deleting old patrons
  let query = r#"
    INSERT INTO
      server_patrons(serverId, userId)
    VALUES
      ($1, $2)
    ON CONFLICT (serverId, userId)
    DO NOTHING
    "#;
  for user_id in user_ids.iter() {
    client.execute(query, &[&server_id, &user_id]).await?;
  }
  Ok(())
}

async fn set_wallet_balance_by_userid(user_id : &str, bal : Numeric, client : &Client) -> StdResult<()> {
  let query = r#"
    INSERT INTO
      wallet (userId, walletBalance)
    VALUES
      ($1, $2)
    ON CONFLICT userId
    DO
      UPDATE SET walletBalance = $2 WHERE userId = $1;
  "#;
  client.execute(query, &[&user_id, &bal]).await?;
  Ok(())
}

async fn get_wallet_balance(wallet_id : i32, client : &Client) -> StdResult<Numeric>{
  let query = r#"
  SELECT walletbalance FROM serverwallets where walletid = $1 LIMIT 1;
  "#;
  let amount : Numeric = client.query_one(query, &[&wallet_id]).await?.try_get("walletbalance")?;
  Ok(amount)
}

async fn update_wallet_balance(wallet_id : i32, transaction : &mut Transaction<'_>) -> StdResult<()> {
  panic!("not implemented");
}

async fn api_keys(client : &Client) -> StdResult<Vec<String>> {
  let query = r#"
    SELECT key_str FROM apikeys
  "#;
  let key_rows = client.query(query, &[]).await?;
  Ok(key_rows.into_iter().map(|row| row.get::<usize, &str>(0).to_string()).collect())
}

impl BrokerMapper {
  
  pub fn new(ds : &DataSource) -> BrokerMapper {
    return BrokerMapper{config : ds.into()};
  }
  
  pub async fn list_currencies(&self) -> StdResult<Vec<CurrencyData>> {
    let currency_list_query = r#"
    with cteLatestPrices as (
      SELECT asOf, 
      symbol, 
      name, 
      price, 
      image_url, 
      market_cap,
      volume,
      coingecko_timestamp,
      ROW_NUMBER() OVER (partition by symbol order by asOf DESC) as rn
      FROM cryptodata
    )
    SELECT * FROM cteLatestPrices 
    WHERE rn = 1 ORDER BY market_cap DESC LIMIT 200;
    "#;
    let conf = self.config.clone();
    let client = get_client!(conf);
    // prob have to spin the connection off
    let mut currency_list = Vec::<CurrencyData>::new();
    for row in client.query(currency_list_query,&[]).await? {
      currency_list.push(CurrencyData::try_from(&row)?);
    }
    Ok(currency_list)
  }

  pub async fn api_keys(&self) -> StdResult<Vec<String>> {
    let config = self.config.clone();
    let client = get_client!(config);
    api_keys(&client).await
  }
  
  pub async fn get_latest_price<S : AsRef<str>>(&self, symbol : S) -> StdResult<Numeric> {
    let query = r#"
    SELECT price FROM cryptodata where LOWER(symbol) = LOWER($1) ORDER BY asOf DESC LIMIT 1;
    "#;
    let config = self.config.clone();
    let client = get_client!(config);
    let price : Numeric = client.query_one(query,&[&symbol.as_ref()]).await?.try_get("price")?;
    Ok(price)
  }
  
  pub async fn get_wallet_balance(&self, wallet_id : i32) -> StdResult<Numeric> {
    let conf = self.config.clone();
    let client = get_client!(conf);
    return get_wallet_balance(wallet_id, &client).await
  }
  
  pub async fn get_wallet_balance_by_userid(&self, user_id : &str) -> StdResult<Numeric> {
    let conf = self.config.clone();
    let client = get_client!(conf);
    return get_wallet_balance_by_userid(user_id, &client).await
  }

  pub async fn set_wallet_balance_by_userid(&self, user_id : &str, bal : Numeric) -> StdResult<()> {
    let conf = self.config.clone();
    let client = get_client!(conf);
    return set_wallet_balance_by_userid(user_id, bal, &client).await
  }
  
  pub async fn buy_currency<S : AsRef<str>>(&self, symbol : S, qty : Numeric, wallet_id : u64) -> StdResult<()> {
    // check have enough money
    panic!("unimplemented");
  }
  
  pub async fn sell_currency<S : AsRef<str>>(&self, symbol : S, qty : Numeric, wallet_id : u64) -> StdResult<()> {
    // check they have enough 
    panic!("unimplemented");
  }
  
  pub async fn get_portfolio(&self, wallet_id : u64) -> StdResult<Portfolio> {
    panic!("unimplemented");
  }
  
  pub async fn update_server_patrons<S : AsRef<str>>(&self, user_ids : &Vec<String>, server_id : S) -> StdResult<()> {
    let conf = self.config.clone();
    let client = get_client!(conf);
    update_server_patrons(user_ids, server_id.as_ref(), &client).await
  }
}


impl TryFrom<&Row> for CurrencyData {
  type Error = tokio_postgres::error::Error;
  
  fn try_from(row : &Row) -> Result<CurrencyData,Self::Error> {
    Ok(CurrencyData {
      as_of : chrono::DateTime::from_utc(row.try_get::<&str,chrono::NaiveDateTime>("asOf")?,chrono::Utc),
      symbol : row.try_get("symbol")?,
      name : row.try_get("name")?,
      price : row.try_get("price")?,
      image_url : row.try_get("image_url")?,
      market_cap : row.try_get("market_cap")?,
      volume: row.try_get("volume")?,
      coingecko_timestamp: row.try_get("coingecko_timestamp")?
    })
  }
}