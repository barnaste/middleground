use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, de};
use serde::de::Visitor;
use std::fmt;
use regex::Regex;

/// A website or book source created by a user
#[derive(Debug)]
pub struct Source {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub credibility: f32,
    pub source_info: SourceInfo,
    pub notes: String,
}

/// Details about a particular website or a list of book matches
#[derive(Debug)]
pub enum SourceInfo {
    Website(WebsiteInfo),
    Book(Vec<BookInfo>),
}

/// Details about a particular website
#[derive(Deserialize, Debug)]
pub struct WebsiteInfo {
    pub url: String,
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub publisher: Option<String>,
    pub date: PublicationDate,
    pub description: Option<String>,
}

/// Details about a particular book
#[derive(Deserialize, Debug)]
pub struct BookInfo {
    pub title: String,
    pub authors: Option<Vec<String>>,
    pub publisher: Option<String>,
    pub date: PublicationDate,
    pub categories: Option<Vec<String>>,
    pub pages: Option<i32>,
}

/// A source publication date consisting of a year, month, and day
#[derive(PartialEq, Debug)]
pub struct PublicationDate {
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

impl Source {
    /// Construct a new Source object (with a new id) given SourceInfo
    pub fn new(source_info: SourceInfo) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            created_by: Uuid::nil(), // TODO: fetch user uuid
            credibility: 0.0, // TODO: implement credibility
            source_info,
            notes: String::new(),
        }
    }
}

impl PublicationDate {
    /// Return an empty PublicationDate object
    fn nil() -> PublicationDate {
        PublicationDate {
            year: None, month: None, day: None
        }
    }

    /// Convert a string of the form '[yyyy][-mm][-dd]' to a PublicationDate object (here, square brackets
    /// mean optional, with the additional rule that if one part is omitted, so must everything to its right).
    /// Returns an PublicationDate::nil() if the string is incorrectly formatted.
    /// This method serves as a helper for PublicationDate's custom Deserializer.
    fn parse_ymd_string(str: &str) -> PublicationDate {
        let re = Regex::new(r"^(?<y>[0-9]{4})?(-(?<m>[0-9]{2}))?(-(?<d>[0-9]{2}))?$").unwrap();

        if let Some(cap) = re.captures_iter(str).last() {
            let mut year = None;
            let mut month = None;
            let mut day = None;

            if let Some(year_match) = cap.name("y") {
                year = Some(year_match.as_str().parse().unwrap());
            }
            if let Some(month_match) = cap.name("m") {
                month = Some(month_match.as_str().parse().unwrap());
            }
            if let Some(day_match) = cap.name("d") {
                day = Some(day_match.as_str().parse().unwrap());
            }
            PublicationDate { year, month, day }
        } else {
            PublicationDate::nil()
        }
    }
}

/// Custom Deserializer for PublicationDate to parse strings of the form '[yyyy][-mm][-dd]'
impl<'de> Deserialize<'de> for PublicationDate {
    fn deserialize<D>(deserializer: D) -> Result<PublicationDate, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct DateVisitor;

        impl<'de> Visitor<'de> for DateVisitor {
            type Value = PublicationDate;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string of the form '[yyyy][-mm][-dd]'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PublicationDate::parse_ymd_string(value))
            }
        }
        
        deserializer.deserialize_string(DateVisitor)
    }
}

/// Unit tests for PublicationDate::parse_ymd_string
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ymd_string_ymd() {
        let input = "2005-03-14";
        let expected = PublicationDate {
            year: Some(2005), month: Some(3), day: Some(14)
        };
        let result = PublicationDate::parse_ymd_string(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_ymd_string_ym() {
        let input = "2005-03";
        let expected = PublicationDate {
            year: Some(2005), month: Some(3), day: None
        };
        let result = PublicationDate::parse_ymd_string(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_ymd_string_y() {
        let input = "2005";
        let expected = PublicationDate {
            year: Some(2005), month: None, day: None
        };
        let result = PublicationDate::parse_ymd_string(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_ymd_string_nil() {
        let input = "";
        let expected = PublicationDate::nil();
        let result = PublicationDate::parse_ymd_string(input);
        assert_eq!(result, expected);
    }
}
