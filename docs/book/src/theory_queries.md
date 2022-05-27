### Queries

Once events are published they can be consumed by our queries (a.k.a., views). As queries consume events they modify 
their own state and produce something similar to the views that we have in standard applications. Their real 
flexibility is derived from the fact that these queries are not tied in any way to our write model.

In the previous example we produced events that we could imagine to be of interest to several queries. Certainly, we 
would have a `customer-information` query that would need to be updated, but then we might have additional queries 
such as an `all-customer-contacts` query that would also respond to the same event.

Additionally, other downstream services may respond to these events similarly to how they would in any other 
messaging-based application. Using the same example we might have a service that sends a verification email to the 
new address after an `email-updated` event is fired.

![CQRS](../../images/CQRS_flow.png)

An example of how a single command might move through a CQRS system.
