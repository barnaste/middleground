//! A source organization and validation system. Uses the Bibify public API to scrape websites and books.

use entity::source::{Source, SourceInfo, WebsiteInfo, BookInfo};
use std::error::Error;

/// Extract source info from a website URL using the Bibify API.
pub async fn extract_source_url(url: &str) -> Result<Source, Box<dyn Error>> {
    let request_target = r#"https://api.bibify.org/api/website"#;
    let query = [("url", url)];

    let client = reqwest::Client::new();

    let request = client.request(reqwest::Method::GET, request_target)
        .query(&query)
        .build()
        .unwrap();

    let response = client.execute(request)
        .await?
        .text()
        .await?;

    let website_info: WebsiteInfo = serde_json::from_str(&response)?;
    let source_info = SourceInfo::Website(website_info);
    let source = Source::new(source_info);

    Ok(source)
}

/// Extract source info from a book by its name using the Bibify API. Returns a list of matches.
pub async fn extract_source_book(name: &str) -> Result<Source, Box<dyn Error>> {
    let request_target = r#"https://api.bibify.org/api/books"#;
    let query = [("q", name)];

    let client = reqwest::Client::new();

    let request = client.request(reqwest::Method::GET, request_target)
        .query(&query)
        .build()
        .unwrap();

    let response = client.execute(request)
        .await?
        .text()
        .await?;

    let book_info: Vec<BookInfo> = serde_json::from_str(&response)?;
    let source_info = SourceInfo::Book(book_info);
    let source = Source::new(source_info);

    Ok(source)
}