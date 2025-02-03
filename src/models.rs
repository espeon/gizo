use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractQuery {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagType {
    #[serde(rename = "name")]
    Name(String),
    #[serde(rename = "property")]
    Property(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTag {
    pub tag_type: TagType,
    pub content: String,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataResponseType {
    OpenGraph(OpenGraphResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataResponse {
    pub metadata: MetadataResponseType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenGraphResponse {
    pub title: String,
    pub og_type: Option<String>,
    pub url: String,
    pub image: Option<String>,
    pub audio: Option<String>,
    pub description: Option<String>,
    pub site_name: Option<String>,
    pub video: Option<String>,
}
impl Default for OpenGraphResponse {
    fn default() -> Self {
        OpenGraphResponse {
            title: String::new(),
            og_type: None,
            url: String::new(),
            image: None,
            audio: None,
            description: None,
            site_name: None,
            video: None,
        }
    }
}
