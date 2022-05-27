## Advanced topics

Running a CQRS application in production provides benefits and concerns different from that in a standard webapp.
These advanced topics cover these additional considerations.

One of the primary reasons for using CQRS with event sourcing is to allow your domain model to change over time.
For changes to the structure or payload of events we use 
[event upcasters](advanced_event_upcasters.md) that translate a persisted older event 
structure into the newer form.

The logic of queries and/or structure of the underlying views may also change over time. 
The approach here is to use an 
[event replay](advanced_event_replay.md) against updated logic to rebuild the persisted views.

Tracking down the origin of state errors can be a notoriously tricky task, particularly if they arise in a
production environment.
Event sourcing allows use to greatly simplify the task of
[debugging state errors](advanced_debugging_state.md) to identify where the problem originates.
