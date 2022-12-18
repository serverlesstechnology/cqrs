use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

use crate::persist::SerializedEvent;

/// Used to upcast and event from an older type or version to the current form. This is needed
/// to modify the structure of events older versions are already persisted.
pub trait EventUpcaster: Send + Sync {
    /// Examines and event type and version to understand if the event should be upcasted.
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool;

    /// Modifies the serialized event to conform the the new structure.
    fn upcast(&self, event: SerializedEvent) -> SerializedEvent;
}

/// A helper type for creating the upcaster function for a `SemanticVersionEventUpcaster`.
pub type SemanticVersionEventUpcasterFunc = dyn Fn(Value) -> Value + Send + Sync;

/// A representation of a semantic version used in a `SemanticVersionEventUpcaster`.
#[derive(Debug, PartialOrd, PartialEq, Eq)]
pub struct SemanticVersion {
    major_version: u32,
    minor_version: u32,
    patch: u32,
}

impl SemanticVersion {
    /// Identifies if one `SemanticVersion` supersedes another. Used to determine whether an
    /// upcaster function should be applied.
    ///
    /// E.g.,
    /// - for upcaster v0.2.3 with code v0.2.2 --> upcaster is applied
    /// - for upcaster v0.2.2 with code v0.2.2 --> upcaster is _not_ applied
    pub fn supersedes(&self, other: &Self) -> bool {
        if other.major_version < self.major_version {
            return true;
        }
        if other.major_version == self.major_version {
            if other.minor_version < self.minor_version {
                return true;
            }
            if other.minor_version == self.minor_version && other.patch < self.patch {
                return true;
            }
        }
        false
    }
}

impl Display for SemanticVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.major_version, self.minor_version, self.patch
        )
    }
}
impl FromStr for SemanticVersion {
    type Err = SemanticVersionError;

    fn from_str(event_version: &str) -> Result<Self, Self::Err> {
        let mut split_version = event_version.split('.').fuse();
        let major_version = u32::from_str(split_version.next().unwrap())?;
        let minor_version = split_version.next().map_or(Ok(0), u32::from_str)?;
        let patch = split_version.next().map_or(Ok(0), u32::from_str)?;
        Ok(Self {
            major_version,
            minor_version,
            patch,
        })
    }
}

#[derive(Debug, PartialOrd, PartialEq, Eq)]
/// Type can not be converted to a `Semantic Version`.
pub struct SemanticVersionError;

impl From<ParseIntError> for SemanticVersionError {
    fn from(_: ParseIntError) -> Self {
        Self
    }
}

/// This upcasts any event that has the same `event_type` and an `event_version` that is less than the
/// version configured on the upcaster.
///
/// ```
/// use cqrs_es::persist::{EventUpcaster,SemanticVersionEventUpcaster};
/// use serde_json::Value;
/// use cqrs_es::persist::SerializedEvent;
///
/// let upcast_function = Box::new(|payload: Value| match payload {
///             Value::Object(mut object_map) => {
///                 object_map.insert("country".to_string(), "USA".into());
///                 Value::Object(object_map)
///             }
///             _ => {
///                 panic!("the event payload is not an object")
///             }
///         });
/// let upcaster = SemanticVersionEventUpcaster::new("EventX", "2.3.4", upcast_function);
///
/// let payload: Value = serde_json::from_str(
///             r#"{
///                     "zip code": 98103,
///                     "state": "Washington"
///                    }"#,
///         ).unwrap();
///  let event = SerializedEvent::new(
///             String::new(),
///             0,
///             String::new(),
///             String::new(),
///             String::new(),
///             payload,
///             Default::default(),
///         );
/// let upcasted_event = upcaster.upcast(event);
///
/// let expected_payload: Value = serde_json::from_str(
///             r#"{
///                     "zip code": 98103,
///                     "state": "Washington",
///                     "country": "USA"
///                    }"#,
///         ).unwrap();
/// let expected_event = SerializedEvent::new(
///             String::new(),
///             0,
///             String::new(),
///             String::new(),
///             "2.3.4".to_string(),
///             expected_payload,
///             Default::default(),
///         );
///
/// assert_eq!(upcasted_event, expected_event);
/// ```
pub struct SemanticVersionEventUpcaster {
    event_type: String,
    event_version: SemanticVersion,
    f: Box<SemanticVersionEventUpcasterFunc>,
}

impl SemanticVersionEventUpcaster {
    /// Creates a `SemanticVersionEventUpcaster`
    pub fn new(
        event_type: &str,
        event_version: &str,
        f: Box<SemanticVersionEventUpcasterFunc>,
    ) -> Self {
        let event_version: SemanticVersion = SemanticVersion::from_str(event_version)
            .expect("event_version is not a valid semantic version");
        Self {
            event_type: event_type.to_string(),
            event_version,
            f,
        }
    }
}

impl EventUpcaster for SemanticVersionEventUpcaster {
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool {
        if event_type != self.event_type {
            return false;
        }
        let event_version = match SemanticVersion::from_str(event_version) {
            Ok(result) => result,
            Err(_) => {
                return false;
            }
        };
        self.event_version.supersedes(&event_version)
    }

    fn upcast(&self, event: SerializedEvent) -> SerializedEvent {
        let upcasted_payload = (self.f)(event.payload);
        SerializedEvent {
            aggregate_id: event.aggregate_id,
            sequence: event.sequence,
            aggregate_type: event.aggregate_type,
            event_type: event.event_type,
            event_version: self.event_version.to_string(),
            payload: upcasted_payload,
            metadata: event.metadata,
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::persist::SerializedEvent;
    use serde_json::json;
    use serde_json::Value;

    use crate::persist::SemanticVersionEventUpcasterFunc;
    use crate::persist::{
        EventUpcaster, SemanticVersion, SemanticVersionError, SemanticVersionEventUpcaster,
    };

    fn semantic_version(major_version: u32, minor_version: u32, patch: u32) -> SemanticVersion {
        SemanticVersion {
            major_version,
            minor_version,
            patch,
        }
    }
    #[test]
    fn parse_version() {
        assert_eq!(
            semantic_version(2, 0, 0),
            SemanticVersion::from_str("2").unwrap()
        );
        assert_eq!(
            semantic_version(2, 3, 0),
            SemanticVersion::from_str("2.3").unwrap()
        );
        assert_eq!(
            semantic_version(2, 3, 4),
            SemanticVersion::from_str("2.3.4").unwrap()
        );
        assert_eq!(
            semantic_version(2, 3, 4),
            SemanticVersion::from_str("2.3.4.5").unwrap()
        );
        assert_eq!(
            Err(SemanticVersionError),
            SemanticVersion::from_str("not_a_version")
        );
    }

    #[test]
    fn simple_upcaster_can_upcast() {
        let upcaster =
            SemanticVersionEventUpcaster::new("EventX", "2.3.4", Box::new(|event| event));
        assert!(upcaster.can_upcast("EventX", "1.12.35"));
        assert!(upcaster.can_upcast("EventX", "2.3.3"));
        assert!(!upcaster.can_upcast("AnotherEvent", "1.12.35"));
        assert!(!upcaster.can_upcast("EventX", "2.3.4"));
        assert!(!upcaster.can_upcast("EventX", "2.3.5"));
        assert!(!upcaster.can_upcast("EventX", "2.4.0"));
        assert!(!upcaster.can_upcast("EventX", "3.0.0"));
    }

    #[test]
    fn semantic_version_upcaster_can_upcast() {
        SemanticVersionEventUpcaster::new("EventX", "2.3.4", test_upcast());
    }

    #[test]
    #[should_panic]
    fn semantic_version_upcaster_invalid_version() {
        SemanticVersionEventUpcaster::new("EventX", "not_a_version", test_upcast());
    }

    #[test]
    fn semantic_version_upcaster_upcast() {
        let upcaster = SemanticVersionEventUpcaster::new("EventX", "2.3.4", test_upcast());
        let payload: Value = serde_json::from_str(
            r#"{
    "id": 4829,
    "name": "George Steinbrenner"
}"#,
        )
        .unwrap();
        let event = SerializedEvent::new(
            String::new(),
            0,
            String::new(),
            String::new(),
            String::new(),
            payload,
            Value::default(),
        );
        println!("{}", event.payload);
        let upcasted_event = upcaster.upcast(event);
        println!("{}", upcasted_event.payload);
    }
    #[test]
    fn semantic_version_upcaster_upcast_for_documentation() {
        let upcast_function = Box::new(|payload: Value| {
            if let Value::Object(mut object_map) = payload {
                object_map.insert("country".to_string(), "USA".into());
                Value::Object(object_map)
            } else {
                panic!("the event payload is not an object")
            }
        });
        let upcaster = SemanticVersionEventUpcaster::new("EventX", "2.3.4", upcast_function);

        let payload: Value = serde_json::from_str(
            r#"{
                    "zip code": 98103,
                    "state": "Washington"
                   }"#,
        )
        .unwrap();
        let event = SerializedEvent::new(
            String::new(),
            0,
            String::new(),
            String::new(),
            String::new(),
            payload,
            Value::default(),
        );
        let upcasted_event = upcaster.upcast(event);

        let expected_payload: Value = serde_json::from_str(
            r#"{
                    "zip code": 98103,
                    "state": "Washington",
                    "country": "USA"
                   }"#,
        )
        .unwrap();
        let expected_event = SerializedEvent::new(
            String::new(),
            0,
            String::new(),
            String::new(),
            "2.3.4".to_string(),
            expected_payload,
            Value::default(),
        );

        assert_eq!(upcasted_event, expected_event);
    }

    fn test_upcast() -> Box<SemanticVersionEventUpcasterFunc> {
        Box::new(|mut payload| {
            let current_id = payload.get("id").unwrap().to_string();
            let updated_id = format!("CUST{}", current_id);
            *payload.get_mut("id").unwrap() = json!(updated_id);
            payload
        })
    }
}
