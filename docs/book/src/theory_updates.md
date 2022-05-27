### Updates
When creating updates under the pattern of CQRS, the focus is solely on what the changes are, and not on what these changes 
mean to any views or queries. This involves requesting the changes via a **_command_** and reflecting the actual 
changes in one or more **_events_**.

To do this, we use a DDD concept called an **_aggregate_**. Roughly, an aggregate combines the state of an object that 
is being acted upon, along with all the business rules of our application. This approach strives to remove any 
technical complexity near the business rules where applications are most sensitive to the errors that degrade their 
agility.

The aggregate’s job is to consider the command in the context of its current state and determine what business facts 
need to be recorded. In effect, a command is a request that is subject to security, validation, application state and 
business rules. The aggregate is the arbiter of these rules.

For instance, if we send an `update-email` command, we would expect an `email-updated` event to be produced by the 
`customer` aggregate.

> Note the difference in naming between a command and an event. A command is a request in the imperative whereas an event 
> is a statement of fact in the past tense.

A single event per command is the most common situation, but this can change based on a number of factors. In the 
event an email address is configured as the customer’s primary contact, we could see a `primary-contact-updated` 
event as well.

Using the same example: If the provided email address is identical to the old email address, then we may not have any 
events at all since there is no change to be reflected. Other situations with no resultant events could be seen for a 
variety of other reasons, such as if the new email address is invalid or if the user requesting the update is not 
authorized to do so.
