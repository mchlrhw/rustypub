use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as Json;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Context {
    Simple(String),
}

impl Default for Context {
    fn default() -> Self {
        Self::Simple("https://www.w3.org/ns/activitystreams".to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    // Activity types.
    Activity,
    IntransitiveActivity,

    Accept,
    Add,
    Announce,
    Arrive,
    Block,
    Create,
    Delete,
    Dislike,
    Flag,
    Follow,
    Ignore,
    Invite,
    Join,
    Leave,
    Like,
    Listen,
    Move,
    Offer,
    Question,
    Reject,
    Read,
    Remove,
    TentativeAccept,
    TentativeReject,
    Travel,
    Undo,
    Update,
    View,

    // Actor types.
    Actor,

    Application,
    Group,
    Organisation,
    Person,
    Service,

    // Object types.
    Object,

    Article,
    Audio,
    Document,
    Event,
    Image,
    Note,
    Page,
    Place,
    Profile,
    Relationship,
    Tombstone,
    Video,

    // Collection types.
    Collection,
    CollectionPage,
    OrderedCollection,
    OrderedCollectionPage,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Object {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub ty: ObjectType,
    #[serde(flatten)]
    extra_fields: Json,
}

impl Object {
    // TODO: Make this fallible.
    pub fn get_field<T: DeserializeOwned>(&self, field: &str) -> Option<T> {
        self.extra_fields
            .as_object()
            .and_then(|o| {
                o.get(field)
                    .map(|v| serde_json::from_value(v.to_owned()).ok())
            })
            .flatten()
    }

    // TODO: Ditto.
    pub fn set_field<T: Serialize>(&mut self, field: &str, value: &T) {
        if let Ok(value) = serde_json::to_value(value) {
            self.extra_fields
                .as_object_mut()
                .and_then(|o| o.insert(field.to_owned(), value));
        }
    }

    // TODO: Ditto.
    pub fn extract<T: DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_value(self.extra_fields.to_owned()).ok()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    Link,
    Mention,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    #[serde(rename = "type")]
    pub ty: LinkType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub href: String,
    #[serde(flatten)]
    extra_fields: Json,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ObjectOrLink {
    Object(Object),
    Link(Link),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonLdDocument {
    #[serde(rename = "@context")]
    pub context: Context,
    #[serde(flatten)]
    pub object: Object,
}

impl Link {
    // TODO: Make this fallible.
    pub fn get_field<T: DeserializeOwned>(&self, field: &str) -> Option<T> {
        self.extra_fields
            .as_object()
            .and_then(|o| {
                o.get(field)
                    .map(|v| serde_json::from_value(v.to_owned()).ok())
            })
            .flatten()
    }

    // TODO: Ditto.
    pub fn set_field<T: Serialize>(&mut self, field: &str, value: &T) {
        if let Ok(value) = serde_json::to_value(value) {
            self.extra_fields
                .as_object_mut()
                .and_then(|o| o.insert(field.to_owned(), value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const EXAMPLE_1: &str = r#"{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Person",
  "id": "https://social.example/alyssa/",
  "name": "Alyssa P. Hacker",
  "preferredUsername": "alyssa",
  "summary": "Lisp enthusiast hailing from MIT",
  "inbox": "https://social.example/alyssa/inbox/",
  "outbox": "https://social.example/alyssa/outbox/",
  "followers": "https://social.example/alyssa/followers/",
  "following": "https://social.example/alyssa/following/",
  "liked": "https://social.example/alyssa/liked/"
}"#;

    const EXAMPLE_16: &str = r#"{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "id": "https://example.net/~mallory/87374",
  "actor": "https://example.net/~mallory",
  "object": {
    "id": "https://example.com/~mallory/note/72",
    "type": "Note",
    "attributedTo": "https://example.net/~mallory",
    "content": "This is a note",
    "published": "2015-02-10T15:04:55Z",
    "to": ["https://example.org/~john/"],
    "cc": ["https://example.com/~erik/followers",
           "https://www.w3.org/ns/activitystreams#Public"]
  },
  "published": "2015-02-10T15:04:55Z",
  "to": ["https://example.org/~john/"],
  "cc": ["https://example.com/~erik/followers",
         "https://www.w3.org/ns/activitystreams#Public"]
}"#;

    #[test]
    fn example_1_roundtrip() -> anyhow::Result<()> {
        let document: JsonLdDocument = serde_json::from_str(EXAMPLE_1)?;

        let serialized = serde_json::to_string(&document)?;

        let deserialized: JsonLdDocument = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized, document);

        Ok(())
    }

    #[test]
    fn example_1_get_field() -> anyhow::Result<()> {
        let document: JsonLdDocument = serde_json::from_str(EXAMPLE_1)?;

        assert_eq!(
            document.object.get_field::<String>("inbox").unwrap(),
            "https://social.example/alyssa/inbox/"
        );

        Ok(())
    }

    #[test]
    fn example_1_set_field() -> anyhow::Result<()> {
        let mut document: JsonLdDocument = serde_json::from_str(EXAMPLE_1)?;

        assert_eq!(
            document.object.get_field::<String>("inbox").unwrap(),
            "https://social.example/alyssa/inbox/"
        );

        let new_val = "https://social.example/brenda/inbox/".to_string();
        document.object.set_field("inbox", &new_val);

        assert_eq!(
            document.object.get_field::<String>("inbox").unwrap(),
            new_val,
        );

        Ok(())
    }

    #[test]
    fn example_16_get_field() -> anyhow::Result<()> {
        let document: JsonLdDocument = serde_json::from_str(EXAMPLE_16)?;
        let inner_object: Object = document.object.get_field("object").unwrap();

        assert_eq!(inner_object.ty, ObjectType::Note);
        assert_eq!(
            inner_object.get_field::<String>("content").unwrap(),
            "This is a note",
        );

        Ok(())
    }

    #[test]
    fn example_16_extract() -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct Inner {
            object: ObjectOrLink,
        }

        let document: JsonLdDocument = serde_json::from_str(EXAMPLE_16)?;
        let inner: Inner = document.object.extract().unwrap();
        let inner_object = match inner.object {
            ObjectOrLink::Object(object) => object,
            _ => panic!(),
        };

        assert_eq!(inner_object.ty, ObjectType::Note);

        Ok(())
    }
}
