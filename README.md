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

##### Update mechanism of order flow
- A pending order event will always fire before an order is able to be filled
- Once an order is filled, a trade event occurs, followed by 2 order events with status filled

#### Bot

TBD