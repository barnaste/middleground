use source::{Source, SourceInfo, WebsiteInfo, BookInfo};
use std::error::Error;

pub mod source;

// Extract source info from a website URL using the Bibify API.
async fn extract_source_url(url: &str) -> Result<Source, Box<dyn Error>> {
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

// Extract source info from a book by its name using the Bibify API. Returns a list of matches.
async fn extract_source_book(name: &str) -> Result<Source, Box<dyn Error>> {
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

    let ds = &mut serde_json::Deserializer::from_str(&response);

    let result: Result<Vec<BookInfo>, _> = serde_path_to_error::deserialize(ds);
    match result {
        Err(err) => {
            let path = err.path().to_string();
            eprintln!("ERROR PATH = {path}");
        }
        _ => ()
    }

    let book_info = serde_json::from_str(&response)?;
    let source_info = SourceInfo::Book(book_info);
    let source = Source::new(source_info);

    Ok(source)
}