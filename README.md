#### Todos:
- Update order functionality
- Optimising cancels for amortised O(1)

### Assumptions:
- Exchange only supports 1 pair
- We have no int overflows
- Orders are received in the same order they are sent, and we do not need unique ids to associate request/responses

### Design Choices 

#### Exchange

For the exchange, I've chosen to use `BTreeSet` for the bids and asks.
- The cancel order mechanism should be optimized with a HashMap, however I've left it as an O(lg N) for now.
- The current exchange does not have an endpoint to propogate orderbook depth at various levels which I would like to add

##### Update mechanism of order flow
- A pending order event will always fire before an order is able to be filled
- Once an order is filled, a trade event occurs, followed by 2 order events with status filled

#### Bot

I haven't had time to flesh out the bot beyond defining its basic attributes and setting up a websocket which polls the Bybit orderbook.

My plan was however to 
1. Use Bybit data as the source of truth 
2. Continously send limit orders to our sandboxed exchange every X seconds, with a slight edge on the highest bid and lowest ask taken from the Bybit book
3. We assume price will generally converge to Bybit price
4. When offloading, increase edge on opposite side of book to incentivise our preferred orders
    - E.g. when we have too much SOL, decrease ask price until we are filled
5. If order isn't filled in X seconds, cancel


#### Simulating

For simulating the sandbox exchange it could make sense to get slightly stale Bybit data (~5-10s) and probabilisticly generate random bids and asks in the book around this price, which our bot can then fill.


#### Testing 

We also need to add unit testing of the exchange and bot structs.