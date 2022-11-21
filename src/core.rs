use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as Json;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Context {
    Simple(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonLdDocument {
    #[serde(rename = "@context")]
    pub context: Context,
    #[serde(flatten)]
    extra_fields: Json,
}

impl JsonLdDocument {
    pub fn get_field<T: DeserializeOwned>(&self, field: &str) -> Option<T> {
        self.extra_fields
            .as_object()
            .and_then(|o| {
                o.get(field)
                    .map(|v| serde_json::from_value(v.to_owned()).ok())
            })
            .flatten()
    }

    pub fn set_field<T: Serialize>(&mut self, field: &str, value: &T) {
        if let Ok(value) = serde_json::to_value(value) {
            self.extra_fields
                .as_object_mut()
                .and_then(|o| o.insert(field.to_owned(), value));
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    Application,
    Group,
    Organisation,
    Person,
    Service,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    #[serde(rename = "type")]
    pub ty: ActorType,
    pub inbox: String,
    pub outbox: String,
    #[serde(flatten)]
    pub extra_fields: Json,
}

impl TryFrom<JsonLdDocument> for Actor {
    type Error = Error;

    fn try_from(doc: JsonLdDocument) -> Result<Self, Self::Error> {
        let mut actor: Self = serde_json::from_value(doc.extra_fields)?;

        if let Json::Object(ref mut map) = actor.extra_fields {
            let context_value = serde_json::to_value(&doc.context)?;
            map.insert("@context".into(), context_value);
        }

        Ok(actor)
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
            document.get_field::<String>("inbox").unwrap(),
            "https://social.example/alyssa/inbox/"
        );

        Ok(())
    }

    #[test]
    fn example_1_into_actor() -> anyhow::Result<()> {
        let document: JsonLdDocument = serde_json::from_str(EXAMPLE_1)?;
        let person: Actor = document.try_into()?;

        assert_eq!(person.inbox, "https://social.example/alyssa/inbox/");

        Ok(())
    }
}
