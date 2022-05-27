## Putting it all together

Now that we have built and tested the logic for our application we need to find a way to utilize it. 
We will start by
building a test application with in-memory persistence in order to understand the fundamentals.
We will need three things for this:

- an event store to insert and retrieve our events
- a query to read our events once committed
- a framework to wire everything together and process commands
