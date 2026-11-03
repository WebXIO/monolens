use mongodb::bson::{self, Bson, Document};
use serde_json::{json, Value};

use crate::drivers::errors::DriverError;

/// Integers beyond this lose precision as JavaScript numbers.
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// A filter, sort or projection from the query tab: an extended JSON object, which bson's
/// serializer turns into BSON types (`{ "$oid": … }` becomes an ObjectId). Null means empty.
pub fn json_to_document(name: &str, value: &Value) -> Result<Document, DriverError> {
    match value {
        Value::Null => Ok(Document::new()),
        Value::Object(_) => bson::to_document(value)
            .map_err(|e| DriverError::InvalidQuery(format!("The {name} could not be read: {e}"))),
        other => Err(DriverError::InvalidQuery(format!(
            "The {name} must be an object like {{ field: value }}, not {}",
            kind_of(other)
        ))),
    }
}

fn kind_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

/// A document for the UI as relaxed extended JSON (dates as ISO strings), but an Int64 beyond
/// ±2^53 stays `{ "$numberLong": "…" }`, which a JavaScript number couldn't hold exactly.
pub fn document_to_json(document: Document) -> Value {
    Value::Object(
        document
            .into_iter()
            .map(|(key, value)| (key, bson_to_json(value)))
            .collect(),
    )
}

fn bson_to_json(value: Bson) -> Value {
    match value {
        Bson::Int64(n) if n.unsigned_abs() > MAX_SAFE_INTEGER => json!({ "$numberLong": n.to_string() }),
        Bson::Document(document) => document_to_json(document),
        Bson::Array(items) => Value::Array(items.into_iter().map(bson_to_json).collect()),
        other => other.into_relaxed_extjson(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::{doc, oid::ObjectId, spec::BinarySubtype, Binary, DateTime, Decimal128};
    use std::str::FromStr;

    #[test]
    fn null_and_empty_objects_are_empty_documents() {
        assert_eq!(json_to_document("filter", &Value::Null).unwrap(), Document::new());
        assert_eq!(json_to_document("filter", &json!({})).unwrap(), Document::new());
    }

    #[test]
    fn extended_json_becomes_bson_types() {
        let document = json_to_document(
            "filter",
            &json!({ "_id": { "$oid": "507f1f77bcf86cd799439011" }, "at": { "$gte": { "$date": "2024-01-01T00:00:00.000Z" } } }),
        )
        .unwrap();

        assert!(matches!(document.get("_id"), Some(Bson::ObjectId(_))));
        assert!(matches!(document.get_document("at").unwrap().get("$gte"), Some(Bson::DateTime(_))));
    }

    #[test]
    fn refuses_values_that_are_not_objects_and_names_the_part() {
        for (value, kind) in [(json!(5), "a number"), (json!([1]), "an array"), (json!("x"), "a string"), (json!(true), "a boolean")] {
            let err = json_to_document("sort", &value).unwrap_err();
            let DriverError::InvalidQuery(message) = &err else { panic!("{err:?}") };
            assert!(message.starts_with("The sort must be an object"), "{message}");
            assert!(message.ends_with(kind), "{message}");
        }
    }

    #[test]
    fn documents_become_relaxed_extended_json() {
        let id = ObjectId::from_str("507f1f77bcf86cd799439011").unwrap();
        let value = document_to_json(doc! {
            "_id": id,
            "name": "Ada",
            "age": 36_i32,
            "visits": 12_i64,
            "score": 1.5,
            "at": DateTime::from_millis(1_704_067_200_000),
            "price": Decimal128::from_str("1.10").unwrap(),
            "tags": ["a", { "n": 1_i32 }],
            "nested": { "deep": { "at": DateTime::from_millis(0) } },
            "bytes": Binary { subtype: BinarySubtype::Generic, bytes: vec![0, 1, 2] },
            "nothing": Bson::Null,
        });

        assert_eq!(
            value,
            json!({
                "_id": { "$oid": "507f1f77bcf86cd799439011" },
                "name": "Ada",
                "age": 36,
                "visits": 12,
                "score": 1.5,
                "at": { "$date": "2024-01-01T00:00:00Z" },
                "price": { "$numberDecimal": "1.10" },
                "tags": ["a", { "n": 1 }],
                "nested": { "deep": { "at": { "$date": "1970-01-01T00:00:00Z" } } },
                "bytes": { "$binary": { "base64": "AAEC", "subType": "00" } },
                "nothing": null,
            })
        );
    }

    #[test]
    fn keeps_large_int64_exact_at_any_depth() {
        let value = document_to_json(doc! {
            "safe": 9_007_199_254_740_991_i64,
            "big": 9_007_199_254_740_993_i64,
            "negative": -9_007_199_254_740_993_i64,
            "list": [9_007_199_254_740_993_i64],
            "nested": { "big": i64::MAX },
        });

        assert_eq!(value["safe"], json!(9_007_199_254_740_991_i64));
        assert_eq!(value["big"], json!({ "$numberLong": "9007199254740993" }));
        assert_eq!(value["negative"], json!({ "$numberLong": "-9007199254740993" }));
        assert_eq!(value["list"][0], json!({ "$numberLong": "9007199254740993" }));
        assert_eq!(value["nested"]["big"], json!({ "$numberLong": i64::MAX.to_string() }));
    }
}
