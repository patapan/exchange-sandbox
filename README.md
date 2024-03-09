Here is a generic sandbox for a mock exchange, with functionality for a basic bot and exchange with empty functions.

Feel free to implement it and change it around as you desire to the architecture / design you desire.

notes:
I've chosen to assume we currently only have 1 pair in the books.

Todos:
- Update order functionality
- Optimising cancels for amortised O(1)

Assumptions
- Assuming no overflows from size
