Here is a generic sandbox for a mock exchange, with functionality for a basic bot and exchange with empty functions.

Feel free to implement it and change it around as you desire to the architecture / design you desire.

notes:
I've chosen to assume we currently only have 1 pair in the books.

Todos:
- Update order functionality
- Optimising cancels for amortised O(1)

Assumptions
- Assuming no overflows from size
- Orders are recieved in the same order they are sent, and we do not need unique ids to associate request/responses


#### Update mechanism of order flow
- A pending order event will always fire before an order is able to be filled
- Once an order is filled, a trade event occurs, followed by 2 order events with status filled
