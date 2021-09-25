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
CREATE TABLE serverWallets (
  walletId SERIAL PRIMARY KEY,
  userId VARCHAR(256),
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
  transactionTime TIMESTAMP NOT NULL,
  walletId INT,--Looks like SERIAL is INT in pg VARCHAR(256),
  cryptoId VARCHAR(256),
  cost NUMERIC(25,4) NOT NULL, -- negative indicates a sell positive indicates a buy
  buySellIndicator CHAR(1) NOT NULL, -- makes life a little bit easier so we dont have to compare cost to 0 to get Buy or Sell
  qty NUMERIC(25,8) NOT NULL,-- amount of crypto purchased / sold positive indicates buy and negative indicates sell
  PRIMARY KEY (transactionId,walletId),
  CONSTRAINT CHK_buySellIndicator CHECK(buySellIndicator in ('B','S')) 
  --This constraint syntax might be for another db software, TODO: fix this
  --CONSTRAINT PK_transaction PRIMARY KEY (transactionId,userId),
  --CONSTRAINT FK_transactionSymbol FOREIGN KEY (symbol) REFERENCES cryptodata(symbol) ON DELETE CASCADE,
  --CONSTRAINT FK_transactionUser FOREIGN KEY (walletId) REFERENCES serverWallets(walletId) ON DELETE CASCADE
);


-- portfolio, and leaderboards should be views since they inherently change all the time and can be calculated from the data above


CREATE VIEW vPortfolio AS
SELECT w.userId, 
t.cryptoId, 
SUM(t.cost) as totalCost,
SUM(t.qty) as qty, 
MAX(t.transactionTime) as lastTransactionTs
FROM transactions t JOIN serverWallets w on t.walletId = w.walletId
GROUP BY 
w.userId,
t.cryptoId;


-- CREATE VIEW vWalletPerformance AS

-- I will finish leaderboards later
