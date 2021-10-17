use tokio_postgres::{Config as PgConfig,Row,NoTls,Client};
use crate::config::DataSource;
use crate::types::*;
use std::convert::TryFrom;
use crate::api::types::*;


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

#[inline(always)]
pub fn push<'a>(base : &str, suffix : &str) -> String {
    let mut heap_base = String::from(base);
    heap_base.push_str(suffix);
    return heap_base;
}

impl BrokerMapper {
  const CTE_LATEST_LIST : &'static str = r#"
  with cteLatestPrices as (
    SELECT asOf,
      id, 
      symbol,  
      name, 
      price, 
      image_url, 
      market_cap,
      volume,
      coingecko_timestamp,
      ROW_NUMBER() OVER (partition by id order by asOf DESC) as rn
    FROM cryptodata
  )
  "#;

  
  pub fn new(ds : &DataSource) -> BrokerMapper {
    return BrokerMapper{config : ds.into()};
  }
  
  pub async fn list_currencies(&self) -> StdResult<Vec<CurrencyData>> {
    let currency_list_query = format!("{} {}",BrokerMapper::CTE_LATEST_LIST,r#"
    SELECT * FROM cteLatestPrices 
    WHERE rn = 1 ORDER BY market_cap DESC LIMIT 200;
    "#);
    let query = currency_list_query.as_str();
    let conf = self.config.clone();
    let client = get_client!(conf);
    // prob have to spin the connection off
    let mut currency_list = Vec::<CurrencyData>::new();
    for row in client.query(query,&[]).await? {
      currency_list.push(CurrencyData::try_from(&row)?);
    }
    Ok(currency_list)
  }

  pub async fn api_keys(&self) -> StdResult<Vec<String>> {
    let config = self.config.clone();
    let client = get_client!(config);
    let query = r#"
    SELECT key_str FROM apikeys
    "#;
    let key_rows = client.query(query, &[]).await?;
    Ok(key_rows.into_iter().map(|row| row.get::<usize, &str>(0).to_string()).collect())
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
  
  pub async fn get_wallet_balance_by_userid<S : AsRef<str>>(&self, user_id : &S) -> StdResult<Numeric> {
    let conf = self.config.clone();
    let client = get_client!(conf);
    let query = r#"
    SELECT walletBalance FROM wallet WHERE userId = $1 LIMIT 1;
    "#;
    let amount : Numeric = client.query_one(query, &[&user_id.as_ref()]).await?.try_get("walletBalance")?;
    return Ok(amount);
  }

  pub async fn set_wallet_balance_by_userid(&self, user_id : &str, bal : Numeric) -> StdResult<()> {
    let conf = self.config.clone();
    let client = get_client!(conf);
    let query = r#"
    INSERT INTO
      wallet (userId, walletBalance)
    VALUES
      ($1, $2)
    ON CONFLICT (userId)
    DO
      UPDATE SET walletBalance = $2;
    "#;
    client.execute(query, &[&user_id, &bal]).await?;
    Ok(())
  }
  
  pub async fn buy_currency<S : AsRef<str>>(&self, crypto_id : &S, qty : &Numeric, user_id : &S) -> StdResult<()> {
    let conf = self.config.clone();
    let client : Client = get_client!(conf);
    client.execute("SELECT * FROM buy_currency($2,$1,$3)", &[&crypto_id.as_ref(),qty,&user_id.as_ref()]).await?;
    Ok(())
  }

  pub async fn get_coins_matching_key(&self, coin_key : &CoinIdentifierKey) -> StdResult<Vec<CurrencyData>> {
    
    let where_conds;
    let param : &String;
    if let Some(crypto_id) = &coin_key.crypto_id {
      where_conds = "id = $1";
      param = crypto_id;
    }
    else if let Some(name) = &coin_key.name {
      where_conds = "lower(name) = lower($1)";
      param = name;
    } 
    else if let Some(symbol) = &coin_key.symbol {
      where_conds = "lower(symbol) = lower($1)";
      param = symbol;
    } else {
      return Err(new_std_err("Please spcify an id, name, or symbol"));
    }
    let query_tail = format!("{} {} AND {}",BrokerMapper::CTE_LATEST_LIST,r#"
    SELECT * FROM cteLatestPrices 
    WHERE rn = 1"#,where_conds);
    let query = query_tail.as_str();
    let config = self.config.clone();
    let client : Client = get_client!(config);
    Ok(
      client.query(query, &[param]).await?
      .iter()
      .map(|r| CurrencyData::try_from(r).expect("Could not map currency data from row"))
      .collect()
    )

  }
  
  #[allow(unused_variables)]
  pub async fn sell_currency<S : AsRef<str>>(&self, crypto_id : &S, qty : &Numeric, user_id : &S) -> StdResult<()> {
    // check they have enough 
    let conf = self.config.clone();
    let client : Client = get_client!(conf);
    client.execute("SELECT * FROM buy_currency($1,$2,$3)", &[qty,&crypto_id.as_ref(),&user_id.as_ref()]).await?;
    Ok(())
  }
  
  pub async fn get_portfolio<S : AsRef<str>>(&self, user_id : &S) -> StdResult<Portfolio> {
    let conf = self.config.clone();
    let client : Client = get_client!(conf);
    let balance = self.get_wallet_balance_by_userid(user_id).await?;
    let positions : Vec<Position> = client.query("SELECT name,cryptoId,currentValue,qty FROM vPortfolio where userId = $1", &[&user_id.as_ref()]).await?.iter().map(|r| Position::try_from(r).expect("Could not create position")).collect();
    Ok(Portfolio{balance,positions})
  }
  
  pub async fn update_server_patrons<S : AsRef<str>>(&self, user_ids : &Vec<String>, server_id : &S) -> StdResult<()> {
    let conf = self.config.clone();
    let client = get_client!(conf);
    // update_server_patrons(user_ids, server_id.as_ref(), &client).await

    let query = r#"
    INSERT INTO
      server_patrons(serverId, userId)
    VALUES
      ($1, $2)
    ON CONFLICT (serverId, userId)
    DO NOTHING
    "#;
    for user_id in user_ids.iter() {
      client.execute(query, &[&server_id.as_ref(), &user_id]).await?;
    }
    Ok(())
  }
}

impl TryFrom<&Row> for CurrencyData {
  type Error = tokio_postgres::error::Error;

  fn try_from(row : &Row) -> Result<CurrencyData,Self::Error> {
    Ok(CurrencyData {
      as_of : chrono::DateTime::from_utc(row.try_get::<&str,chrono::NaiveDateTime>("asOf")?,chrono::Utc),
      id : row.try_get("id")?,
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

impl TryFrom<&Row> for Position {
  type Error = tokio_postgres::error::Error;
  fn try_from(row : &Row) -> Result<Position,Self::Error> {
    Ok(Position{
      name : row.try_get("name")?,
      crypto_id : row.try_get("cryptoId")?,
      current_value : row.try_get("currentValue")?,
      qty: row.try_get("qty")?
    })
  }
}
