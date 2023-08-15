/*
 * Apache Iceberg REST Catalog API
 *
 * Defines the specification for the first version of the REST Catalog API. Implementations should ideally support both Iceberg table specs v1 and v2, with priority given to v2.
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Term {
    #[serde(rename = "type")]
    pub r#type: RHashType,
    #[serde(rename = "transform")]
    pub transform: String,
    #[serde(rename = "term")]
    pub term: String,
}

impl Term {
    pub fn new(r#type: RHashType, transform: String, term: String) -> Term {
        Term {
            r#type,
            transform,
            term,
        }
    }
}

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum RHashType {
    #[serde(rename = "transform")]
    Transform,
}

impl Default for RHashType {
    fn default() -> RHashType {
        Self::Transform
    }
}