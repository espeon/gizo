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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardyBResponse {
    pub error: String,
    pub likely_type: String,
    pub url: String,
    pub title: String,
    pub description: String,
    pub image: String,
}

impl From<OpenGraphResponse> for CardyBResponse {
    fn from(og: OpenGraphResponse) -> Self {
        // we NEED url and title
        if og.url.is_empty() || og.title.is_empty() {
            return CardyBResponse {
                error: "Unable to generate link preview".to_string(),
                likely_type: "".to_string(),
                url: "".to_string(),
                title: "".to_string(),
                description: "".to_string(),
                image: "".to_string(),
            };
        }
        CardyBResponse {
            error: "".to_string(),
            likely_type: og.og_type.unwrap_or("website".to_string()),
            url: og.url,
            title: og.title,
            description: og.description.unwrap_or("".to_string()),
            image: og.image.unwrap_or("".to_string()),
        }
    }
}
