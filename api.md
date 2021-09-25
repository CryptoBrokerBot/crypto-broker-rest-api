# Crypto Bot API

All endpoints need an API key header called CB-Api-Key

---
## GET /list
*Lists top 200 crypto currencies by market cap*
```ts
[
  {
    "symbol": string,
    "name": string,
    "price": number
  }
]
```
Status Codes
---

## POST /buy
*Buys a new cryptocurrency by symbol or name*
```ts
[
  {
    "symbol" | "name" : string,
    "quantity": number,
    "user": string
  }
]
```
Status Codes
---

## POST /sell
*Sells a new cryptocurrency by symbol or name*

#### Request
```ts
[
  {
    "symbol": string,
    "name": string,
    "quantity": number,
    "userId": string
  }
]
```
Status Codes
---

## GET /portfolio
`userId` : string

*Gets the users portfolio and current balance*
### Response
```ts
interface Position {
  "name" : string,
  "symbol" : string,
  "currentValue" : number,
  "qty" : number
};
// Returns
{
  "balance" : number,
  "positions" : Position[]
}
```
Status Codes 
---

## PUT /leaderboard

*Updates the the users on a leaderboard*

```ts 
{
  "serverId": string,
  "users": {user_id : string, netWorth : number}[]
}
```
Status Codes
---

## GET /leaderboard?serverId=serverId
`serverId` : string

*Gets the top 10 users sorted by their balance*

```ts 
interface LeaderboardEntry {
  "userId": string,
  "balance": number
};

{
  "entries": LeaderboardEntry[]
}
```

## GET /graph/performance
`userId`: string

`dateRange`: "month" | "day" | "week" | "90days" | "6months"

*Returns a PNG with a graph of their performance*

## GET /graph/leaderboard
`serverId`: string

`dateRange`: "month" | "date" | "week" | "90days" | "6months"

*Returns a PNG with a comparison of user's performance on a leaderboard*

## GET /graph/coin
`symbol` | `name` : string

`dateRange`: "month" | "date" | "week" | "90days" | "6months"

*Returns a PNG with a coin's performance history*