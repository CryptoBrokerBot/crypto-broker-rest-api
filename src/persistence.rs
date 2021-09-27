use tokio_postgres::{Config as PgConfig,Row,NoTls,Client};
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
        ROW_NUMBER() OVER (partition by id order by asOf DESC) as rn
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
    SELECT walletBalance FROM wallets WHERE userId = $1 LIMIT 1;
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
    ON CONFLICT userId
    DO
      UPDATE SET walletBalance = $2 WHERE userId = $1;
    "#;
    client.execute(query, &[&user_id, &bal]).await?;
    Ok(())
  }
  
  pub async fn buy_currency<S : AsRef<str>>(&self, crypto_id : S, qty : Numeric, user_id : S) -> StdResult<()> {
    // check have enough money
    let query = r#"
    do $$
    declare wbal numeric(50,10) := 0.0;
    declare qty NUMERIC(50,10) := $2;
    declare currentPrice NUMERIC(50,10) := NULL;
    declare l_cryptoId VARCHAR(32) := $1;
    declare l_userId VARCHAR(256) := $3;
    begin 
    if (qty <= 0) then 
	    raise exception 'Qty must be a positive decimal!';
    end if;
    -- fetch current price
    select price into currentPrice from cryptodata c where c.id = l_cryptoId order by asOf desc limit 1;
    if (currentPrice is null or currentPrice <= 0.0) then 
    	  raise exception 'Could not find a nonzero price for %',l_cryptoId;
    end if;
   
    -- explicitly lock the the wallet table in row exclusive mode
    lock table wallet in row exclusive mode;
    select w.walletbalance into wbal from wallet w
    where w.userId = l_userId for update;
   
    -- make sure they have enough money
    if (wbal is null or (wbal - qty*currentPrice) < 0.0) then
         raise exception 'Insufficient funds';
    end if;
   
    -- update wallet balance
    update wallet set walletbalance = (wbal - qty*currentPrice);
    -- create the transaction
    insert into transactions (userId,cryptoid,cost,buysellindicator,qty) 
    values (l_userId,l_cryptoId,qty*currentPrice,'B',qty);
    end $$;
    select lastval() as transactionId;
    "#;
    let conf = self.config.clone();
    let client : Client = get_client!(conf);
    client.execute(query, &[&crypto_id.as_ref(),&qty,&user_id.as_ref()]).await?;
    Ok(())
  }
  
  #[allow(unused_variables)]
  pub async fn sell_currency<S : AsRef<str>>(&self, crypto_id : S, qty : Numeric, user_id : S) -> StdResult<()> {
    // check they have enough 
    panic!("unimplemented");
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