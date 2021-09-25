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


-- Wallet for each user for each server, each user can participate in multiple servers.
CREATE TABLE wallet (
  userId VARCHAR(256) PRIMARY KEY,
  walletBalance NUMERIC(25,4)
);


CREATE TABLE serverpatrons (
  serverId VARCHAR(256),
  userId VARCHAR(256),
  ts TIMESTAMP NOT NULL DEFAULT NOW()
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
  SELECT id as cryptoId, price as latestPrice FROM (SELECT id, price, ROW_NUMBER() OVER(partition by id order by asOf DESC) as rn
  FROM cryptodata) res WHERE res.rn = 1
),
ctePositions AS (
  SELECT w.userId, 
  t.cryptoId, 
  SUM(t.qty) as qty,
  MAX(t.transactionTime) as lastTransactionTs
  FROM transactions t 
  JOIN wallet w on t.userId = w.userId
  GROUP BY 
    w.userId,
    t.cryptoId
)
SELECT userId,
p.cryptoId, 
qty*latestPrice as currentValue,
lastTransactionTs
FROM ctePositions p
JOIN cteLatestPrices lp on p.cryptoId = lp.cryptoId
-- CREATE VIEW vWalletPerformance AS

-- I will finish leaderboards later