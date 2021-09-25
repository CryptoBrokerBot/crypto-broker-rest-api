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

macro_rules! result_err {
  ($msg : expr) => {
    {
      use std::io::{ErrorKind,Error as StdIoErr};
      Err(Box::new(StdIoErr::new(ErrorKind::Other,$msg)))
    }
  };
}


async fn get_wallet_balance<S : AsRef<str>>(user_id : S, client : &Client) -> StdResult<Numeric>{
    let query = r#"
    SELECT walletbalance FROM serverwallets where user_id = $1 LIMIT 1;
    "#;
    let amount : Numeric = client.query_one(query, &[&user_id.as_ref()]).await?.try_get("walletbalance")?;
    Ok(amount)
}

async fn update_wallet_balance<S : AsRef<str>>(user_id : S, balance : Numeric, transaction : &mut Transaction<'_>) -> StdResult<()> {
    let affected = transaction.execute("UPDATE serverwallets SET walletbalance = $1",&[&balance]).await?;
    if affected == 0 {
      return result_err!("Could not find wallet for user!");
    }
    Ok(())
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

  pub async fn get_latest_price<S : AsRef<str>>(&self, symbol : S) -> StdResult<Numeric> {
    let query = r#"
    SELECT price FROM cryptodata where LOWER(symbol) = LOWER($1) ORDER BY asOf DESC LIMIT 1;
    "#;
    let config = self.config.clone();
    let client = get_client!(config);
    let price : Numeric = client.query_one(query,&[&symbol.as_ref()]).await?.try_get("price")?;
    Ok(price)
  }

  pub async fn get_wallet_balance<S : AsRef<str>>(&self, user_id : S) -> StdResult<Numeric> {
    let conf = self.config.clone();
    let client = get_client!(conf);
    return get_wallet_balance(user_id, &client).await
  }

  // async fn insert_transaction(user_id : S, buy_sell : BuySellIndicator, )

  pub async fn buy_currency<S : AsRef<str>>(&self, symbol : S, qty : Numeric, user_id : S) -> StdResult<()> {
    // check have enough money
    assert!(qty > Numeric::ZERO);
    let conf = self.config.clone();
    let mut client : Client = get_client!(conf);
    let balance = get_wallet_balance(user_id, &client).await?;
    let current_cost = self.get_latest_price(symbol).await?;
    if balance <= Numeric::ZERO || qty*current_cost > balance {
      return result_err!("User does not have enough money in their wallet. Please sell a position or add some money!");
    }
    // create a new transaction
    let tx = client.transaction().await?;
    let new_balance = balance - (qty*current_cost);
    // update_wallet_balance(user_id, )
    panic!("unimplemented");
  }

  pub async fn sell_currency<S : AsRef<str>>(&self, symbol : S, qty : Numeric, wallet_id : u64) -> StdResult<()> {
    // check they have enough 
    panic!("unimplemented");
  }

  pub async fn get_portfolio<S : AsRef<str>>(&self, user_id : S) -> StdResult<Portfolio> {
    let conf = self.config.clone();
    let client : Client = get_client!(conf);
    let positions : Vec<Position> = 
    client.query("SELECT name,qty,symbol,totalcost FROM vpositions where userId = $1",&[&user_id.as_ref()]).await?
          .iter()
          .map(|r| Position::try_from(r).expect("Could not construct a position."))
          .collect();
    let balance = get_wallet_balance(user_id, &client).await?;
    Ok(Portfolio{
        balance,positions
    })
  }

  pub async fn update_server_patrons<S : AsRef<str>>(&self, user_id : S, server_id : S) -> StdResult<()> {
    unimplemented!("please implement updating server patrons");
  }

  pub async fn create_wallet<S : AsRef<str>>(&self, user_id : S, balance : Numeric) -> StdResult<()> {
    let query = r#"
        INSERT INTO serverwallets (userId, walletbalance) VALUES($1,$2);
    "#;
    let config = self.config.clone();
    let client : Client = get_client!(config);
    let _ = client.execute(query, &[&user_id.as_ref(),&balance]).await?;
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
        qty : row.try_get("qty")?,
        symbol : row.try_get("symbol")?,
        total_cost : row.try_get("totalCost")?
    })
  }
}