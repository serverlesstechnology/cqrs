## Event Upcasting

Over time the domain model will need to be modified to adapt to new business rules,
and with event sourcing the domain model directly relates to events. 
Event changes can be minimized by keeping events small and focused, but they will be needed.
This can be a challenge because domain events are append-only and immutable.

As an example, if our bank services only local customers there is no need to identify the state as part of their address,
this is understood. The payload for an `UpdateAddress` event might look something like:
```json
{
  "UpdateAddress": {
    "address": "912 Spring St",
    "city": "Seattle"
  }
}
```

If however the bank begins servicing customers in other states we'll need additional information in our payload, e.g.,
```json
{
  "UpdateAddress": {
    "address": "912 Spring St",
    "city": "Seattle",
    "state": "WA"
  }
}
```

We are event sourced, so we will need to load past events in order to build our aggregate to process new commands.
However, the persisted form of the event no longer matches the new structure.

We can leverage the features of [serde](https://serde.rs/) to allow backwards compatibility with persisted events.

### Minor Changes

Adding new fields to event structures is backwards-compatible, so long as the new fields are optional, or the `serde` implementation provides a default value for the field.

For example,
```rust
#[derive(Serialize, Deserialize)]
pub struct UpdateAddress {
  address: String,
  city: String,
  state: Optional<String>,
}
```

or

```rust
#[derive(Serialize, Deserialize)]
pub struct UpdateAddress {
  address: String,
  city: String,
  #[serde(default)]
  state: String,
}
```

### Major Changes

Sometimes changes to an event's structure cannot be made backwards-compatible through default or optional values. Such changes can include changing a field's type, or reusing the name of a legacy field for a new purpose.

It is often beneficial to separate the internal representation of the event, and its serialised representation. This allows the two representations to evolve independently. Serde can convert (either fallibly or infallibly) between the two representations during serialisation/deserialisation.

The following is a more complex example which demonstrates how these serde features can be leveraged to introduce backwards compatibility.

```rust
use serde::{Deserialize, Serialize};

/// The event we wish to serialize.
/// 
/// Note: if conversions from `rep::Rep` were fallible, we could use-
/// `#[serde(try_from = "rep::Rep", into = "rep::Rep")]`
#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "rep::Rep", into = "rep::Rep")]
pub struct Event {
    /// Customer name
    customer: String,
    /// Customer ID
    customer_id: String,
}

mod rep {
    //! The serialised representation of the [`Event`].
    //!
    //! Includes previous versions of the representation, for backwards compatibility
    use serde::{Deserialize, Serialize};

    use super::Event;

    /// The serialised representation of the [`Event`].
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "version", content = "payload")]
    pub enum Rep {
        #[serde(rename = "1")]
        V1(V1),
        #[serde(rename = "2")]
        V2(V2),
    }

    /// Version 1 of the event's serialised representation
    #[derive(Serialize, Deserialize)]
    pub struct V1 {
        /// Customer ID
        customer: String,
    }

    /// Version 2 of the event's serialised representation
    ///
    /// In version '2' of this event's serialised representation, the 'customer' field now refers to the customer's name, rather than their ID.
    #[derive(Serialize, Deserialize)]
    pub struct V2 {
        /// Customer name
        customer: String,
        /// Customer ID
        customer_id: String,
    }

    // In this case, v1 of the event can be converted infallibly to v2 of the event.
    //
    // If this wasn't the case, we could use [`TryFrom`] instead.
    impl From<V1> for V2 {
        fn from(v1: V1) -> Self {
            Self {
                customer: String::new(),
                customer_id: v1.customer,
            }
        }
    }

    impl From<V1> for super::Event {
        fn from(event: V1) -> Self {
            Self {
                customer: String::new(),
                customer_id: event.customer,
            }
        }
    }

    impl From<V2> for super::Event {
        fn from(event: V2) -> Self {
            Self {
                customer: event.customer,
                customer_id: event.customer_id,
            }
        }
    }

    impl From<Rep> for Event {
        fn from(versions: Rep) -> Self {
            match versions {
                Rep::V1(event) => event.into(),
                Rep::V2(event) => event.into(),
            }
        }
    }

    impl From<Event> for Rep {
        fn from(event: Event) -> Self {
            Self::V2(V2 {
                customer: event.customer,
                customer_id: event.customer_id,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let input = r#"{
    "version": "1",
    "payload": {
        "customer": "ID-001"
    }
}"#;

        let event: Event = serde_json::from_str(input).unwrap();
        let actual = serde_json::to_string_pretty(&event).unwrap();

        let expected = r#"{
  "version": "2",
  "payload": {
    "customer": "",
    "customer_id": "ID-001"
  }
}"#;

        assert_eq!(expected, actual);
    }
}
```
