use crate::{
    error::AppError,
    models::{MetaTag, MetadataResponse, MetadataResponseType, OpenGraphResponse, TagType},
};
use regex::Regex;

lazy_static::lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(
        r#"(https?:\/\/)?(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)"#
    ).unwrap();
}

pub async fn extract_metadata(url: &str) -> Result<MetadataResponse, AppError> {
    if !URL_REGEX.is_match(url) {
        return Err(AppError::InvalidUrl);
    }

    let headers = get_url_headers(url).await?;
    let tags = get_open_graph_tags(&headers);
    let og_response = collate_og_tags(tags);

    Ok(MetadataResponse {
        metadata: MetadataResponseType::OpenGraph(og_response),
    })
}

pub async fn get_url_headers(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(
            "user-agent",
            "Mozilla/5.0 (compatible; richardbot/1.0; +https://example.com)",
        )
        .send()
        .await?;
    let text = res.text().await?;
    // get head tag
    let mut head = text.split("head>").collect::<Vec<&str>>()[1].to_string();
    // remove </ at the end
    let mut mid = head.split("</").collect::<Vec<&str>>();
    mid.pop();
    head = mid.join("</");
    Ok(head.to_string())
}

/// Extract all open graph meta tags from passed-in header string
pub fn get_open_graph_tags(header: &str) -> Vec<MetaTag> {
    let mut meta_tags = Vec::new();
    let mut tag = String::new();

    // Wildcard match string to match any valid stripped head tag.
    // For example, for meta name="og:title" content="The Beatles - Hey Jude"
    // the captures would be: ["name", "og:title", "The Beatles - Hey Jude"]

    for c in header.chars() {
        if c == '<' {
            if tag.contains("meta") || tag.contains("link") {
                meta_tags.push("<".to_string() + &tag.clone());
            }
            tag = String::new();
        } else {
            tag.push(c);
        }
    }
    dbg!(&meta_tags);
    // parse all tags
    let mut tags = Vec::new();
    for tag in meta_tags {
        // TODO: handle oembed tags separately
        if !(tag.contains("name=") || tag.contains("property=") || tag.contains("itemprop=")) {
            //println!("skipping tag: {}", tag);
            continue;
        }
        let re =
            Regex::new(r#"<(meta|link)\s+(?:(?:name|property|itemprop|rel)="([^"]*)")(?:\s+(?:content|href)="([^"]*)")"#)
                .unwrap();
        if let Some(captures) = re.captures(&tag) {
            let prefix = if captures[0].to_string() == "link" {
                "link:"
            } else {
                ""
            };
            let tag_type = TagType::Name(prefix.to_string() + &captures[2]);

            let content = captures[3].to_string();
            let raw = tag.to_string();
            tags.push(MetaTag {
                tag_type,
                content,
                raw,
            });
        }
        // else {
        //     println!("failed to parse tag: {}", tag);
        // }
    }

    tags
}

pub fn collate_og_tags(tags: Vec<MetaTag>) -> OpenGraphResponse {
    let mut response = OpenGraphResponse::default();
    for tag in &tags {
        match &tag.tag_type {
            TagType::Name(name) => match name.as_str() {
                "og:title" | "twitter:title" => response.title = tag.content.clone(),
                "og:type" => response.og_type = Some(tag.content.clone()),
                "og:url" => response.url = tag.content.clone(),
                "og:image" => response.image = Some(tag.content.clone()),
                "og:audio" => response.audio = Some(tag.content.clone()),
                "og:description" | "twitter:description" => {
                    response.description = Some(tag.content.clone())
                }
                "og:site_name" => response.site_name = Some(tag.content.clone()),
                "og:video" => response.video = Some(tag.content.clone()),
                _ => {}
            },
            TagType::Property(property) => match property.as_str() {
                "og:title" => response.title = tag.content.clone(),
                "og:type" => response.og_type = Some(tag.content.clone()),
                "og:url" => response.url = tag.content.clone(),
                "og:image" => response.image = Some(tag.content.clone()),
                "og:audio" => response.audio = Some(tag.content.clone()),
                "og:description" => response.description = Some(tag.content.clone()),
                "og:site_name" => response.site_name = Some(tag.content.clone()),
                "og:video" => response.video = Some(tag.content.clone()),
                _ => {}
            },
        };
    }
    // site-specific overrides

    // YouTube (videos and shorts)
    if response.url.starts_with("https://youtube.com/watch")
        || response.url.starts_with("https://www.youtube.com/watch")
        || response.url.starts_with("https://youtube.com/shorts")
        || response.url.starts_with("https://www.youtube.com/shorts")
    {
        // get link:name tag
        let mut link_name = String::new();
        for tag in tags {
            if let TagType::Name(name) = tag.tag_type {
                if name.starts_with("name") {
                    link_name = tag.content;
                }
            }
        }
        let item_type = if response.url.contains("/shorts") {
            "short"
        } else {
            "video"
        };
        response.description = Some(format!("Youtube {} by {}", item_type, &link_name));
    }

    // convert image to proxy url
    if let Some(image) = &response.image {
        // todo: not hardcode this
        let base_url = std::env::var("BASE_URL").unwrap_or("https://cardyb.bsky.app".to_string());
        response.image = Some(format!(
            "{base_url}/v1/image?url={}",
            &urlencoding::encode(image)
        ));
    }
    response
}

#[cfg(test)]
mod tests {
    use crate::models::CardyBResponse;

    use super::*;

    #[test]
    fn test_get_open_graph_tags() {
        let header = r#"<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="description" content="The Beatles - Hey Jude">
    <meta property="og:title" content="The Beatles - Hey Jude">
    <meta property="og:type" content="music.song">
    <meta property="og:url" content="https://music.apple.com/us/album/hey-jude/1435546686">
    <meta property="og:image" content="https://is3-ssl.mzstatic.com/image/thumb/Music128/v4/c8/b8/f5/c8b8f5d0-b5a6-c5e1-a5f9-f7e8f2e6f0e2/88229964-e0b9-4c3d-8b6c-d2f8c9f3a2d8.jpg/170x170bb.jpg">
    <meta property="og:audio" content="https://is3-ssl.mzstatic.com/audio/v4/c8/b8/f5/c8b8f5d0-b5a6-c5e1-a5f9-f7e8f2e6f0e2/mzaf_1435546686.mp3/mzaf_1435546686.mp3">
    <meta property="og:description" content="The Beatles - Hey Jude">
    <meta property="og:site_name" content="Apple Music">
    <meta property="og:video" content="https://is3-ssl.mzstatic.com/video/thumb/Music128/v4/c8/b8/f5/c8b8f5d0-b5a6-c5e1-a5f9-f7e8f2e6f0e2/mzaf_1435546686.mp4/mzaf_1435546686.mp4">
        </head>
        <body>
        </body>
        </html>"#;
        let tags = get_open_graph_tags(header);
        assert_eq!(tags.len(), 10);

        let collated = collate_og_tags(tags);
        assert_eq!(collated.title, "The Beatles - Hey Jude");
        assert_eq!(
            collated.url,
            "https://music.apple.com/us/album/hey-jude/1435546686"
        );
        assert_eq!(collated.image, Some("https://cardyb.bsky.app/v1/image?url=https%3A%2F%2Fis3-ssl.mzstatic.com%2Fimage%2Fthumb%2FMusic128%2Fv4%2Fc8%2Fb8%2Ff5%2Fc8b8f5d0-b5a6-c5e1-a5f9-f7e8f2e6f0e2%2F88229964-e0b9-4c3d-8b6c-d2f8c9f3a2d8.jpg%2F170x170bb.jpg".to_string()));

        // to cardyb
        let collated: CardyBResponse = collated.into();
        assert_eq!(collated.likely_type, "music.song");
        assert_eq!(
            collated.url,
            "https://music.apple.com/us/album/hey-jude/1435546686"
        );
        assert_eq!(collated.title, "The Beatles - Hey Jude");
    }
}
