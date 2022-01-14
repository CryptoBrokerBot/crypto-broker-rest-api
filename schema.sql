CREATE TABLE cryptodata (
  id VARCHAR(256) NOT NULL,
  asOf TIMESTAMP NOT NULL DEFAULT NOW(), -- the datetime which this quote was pulled from coingecko
  symbol VARCHAR(32) NOT NULL,
  name VARCHAR(256) NOT NULL,
  price NUMERIC(50,10) NOT NULL,
  image_url VARCHAR(512),
  market_cap NUMERIC(50, 4) NOT NULL,
  volume NUMERIC(40, 0) NOT NULL,
  coingecko_timestamp VARCHAR(128) NOT NULL,
  CONSTRAINT PK_cryptodata PRIMARY KEY (asOf,id) -- time series lookups will be fast now :)
);

CREATE INDEX IDX_cryptoname ON cryptodata(name); -- name lookups are now fast :)
CREATE INDEX IDX_cryptoasof ON cryptodata(asOf); 

-- Wallet for each user for each server, each user can participate in multiple servers.
CREATE TABLE wallet (
  userId VARCHAR(256) PRIMARY KEY,
  walletBalance NUMERIC(25,4)
);

CREATE TABLE apikeys (
  id SERIAL PRIMARY KEY,
  key_str VARCHAR(512),
  description VARCHAR(1024)
);

CREATE TABLE serverpatrons (
  serverId VARCHAR(256),
  userId VARCHAR(256),
  ts TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(serverId, userId)
);
CREATE INDEX IDX_serverpatrons_serverId on serverpatrons(serverId);

-- transactions belong to a serverwallet (which is unique to a (userId,serverId) combo)
CREATE TABLE transactions (
  transactionId SERIAL,
  transactionTime TIMESTAMP NOT NULL DEFAULT NOW(),
  userId VARCHAR(256),--Looks like SERIAL is INT in pg VARCHAR(256),
  cryptoId VARCHAR(256),
  cost NUMERIC(25,4) NOT NULL, -- negative indicates a sell positive indicates a buy
  buySellIndicator CHAR(1) NOT NULL, -- makes life a little bit easier so we dont have to compare cost to 0 to get Buy or Sell
  qty NUMERIC(25,8) NOT NULL,-- amount of crypto purchased / sold positive indicates buy and negative indicates sell
  PRIMARY KEY (transactionId),
  CONSTRAINT CHK_buySellIndicator CHECK(buySellIndicator in ('B','S')),
  constraint CHK_cost CHECK( (cost > 0 and qty > 0 and buySellIndicator = 'B') or (cost < 0 and qty < 0 and buySellIndicator = 'S'))
);

create index IDX_transactions on transactions (userId,cryptoId);


-- portfolio, and leaderboards should be views since they inherently change all the time and can be calculated from the data above

CREATE VIEW vPortfolio AS
WITH cteLatestPrices AS (
  SELECT id as cryptoId, symbol, name, price as latestPrice FROM (SELECT id, price, symbol, name, ROW_NUMBER() OVER(partition by id order by asOf DESC) as rn
  FROM cryptodata) res WHERE res.rn = 1
),
ctePositions AS (
  SELECT t.userId, 
  t.cryptoId, 
  SUM(t.qty) as qty,
  MAX(t.transactionTime) as lastTransactionTs
  FROM transactions t 
  GROUP BY 
    t.userId,
    t.cryptoId
)
SELECT userId,
p.cryptoId,
lp.symbol,
lp.name,
qty,
latestPrice,
qty*latestPrice as currentValue,
lastTransactionTs
FROM ctePositions p
JOIN cteLatestPrices lp on p.cryptoId = lp.cryptoId
WHERE qty > 0;

create view vNetworth AS
with ctePortfolioValue as (
	select userId, SUM(currentValue) as portfolioValue from vportfolio v group by userid
)
select w.userid, coalesce(pv.portfolioValue,0.0)+w.walletbalance as netWorth 
from wallet w left join ctePortfolioValue pv on w.userid = pv.userId



create function buy_currency(qty numeric(50,10), l_cryptoId VARCHAR(32), l_userId VARCHAR(256)) returns void AS
 $BODY$
declare wbal numeric(50,10) := 0.0;
declare currentPrice NUMERIC(50,10) := NULL;
BEGIN
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
end $BODY$
 LANGUAGE 'plpgsql' 
COST 100;

-- create sell_currency fn
create function sell_currency(qty numeric(50,10), l_cryptoId VARCHAR(32), l_userId VARCHAR(256)) returns void AS
 $BODY$
declare ownedAmnt numeric(50,10) := 0.0;
declare currentPrice NUMERIC(50,10) := NULL;
BEGIN
if (qty <= 0) then 
 raise exception 'Qty must be a positive decimal!';
end if;
-- fetch current price
select price into currentPrice from cryptodata c where c.id = l_cryptoId order by asOf desc limit 1;
if (currentPrice is null or currentPrice <= 0.0) then 
	  raise exception 'Could not find a nonzero price for %',l_cryptoId;
end if;
-- explicitly lock the the transactions table to prevent overselling
lock table transactions;

select SUM(t.qty) into ownedAmnt from transactions t where t.userid = l_userId and t.cryptoid = l_cryptoId;
-- make sure they have enough coin
if (ownedAmnt is null or ownedAmnt < qty) then
     raise exception 'Insufficient funds';
end if;
-- update wallet balance
update wallet set walletbalance = (walletbalance + qty*currentPrice);
-- create the transaction
insert into transactions (userId,cryptoid,cost,buysellindicator,qty) 
values (l_userId,l_cryptoId,-1*qty*currentPrice,'S',-1*qty);
end $BODY$
 LANGUAGE 'plpgsql' 
COST 100;

-- I will finish leaderboards later