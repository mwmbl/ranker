mod utils;

use arrayvec::ArrayString;
use std::cmp::max;
use std::collections::HashMap;
use url::Url;
use wasm_bindgen::prelude::*;

const MAX_URL_LENGTH: usize = 150;
const MAX_TITLE_LENGTH: usize = 65;
const MAX_EXTRACT_LENGTH: usize = 155;

const MISSING_URL: &str = "https://_.com";

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, ranker!");
}

struct SearchResult {
    pub url: ArrayString<MAX_URL_LENGTH>,
    pub title: ArrayString<MAX_TITLE_LENGTH>,
    pub extract: ArrayString<MAX_EXTRACT_LENGTH>,
}

impl SearchResult {
    pub fn new(url: &str, title: &str, extract: &str) -> SearchResult {
        SearchResult {
            url: ArrayString::from(if url.len() > MAX_URL_LENGTH {
                &url[..MAX_URL_LENGTH]
            } else {
                url
            })
            .unwrap(),
            title: ArrayString::from(if title.len() > MAX_TITLE_LENGTH {
                &url[..MAX_TITLE_LENGTH]
            } else {
                title
            })
            .unwrap(),
            extract: ArrayString::from(if extract.len() > MAX_EXTRACT_LENGTH {
                &url[..MAX_EXTRACT_LENGTH]
            } else {
                extract
            })
            .unwrap(),
        }
    }
}

#[wasm_bindgen]
struct Ranker {
    query: String,
    search_results: Vec<SearchResult>,
}

#[wasm_bindgen]
impl Ranker {
    pub fn new(query: &str) -> Ranker {
        Ranker {
            query: query.to_string(),
            search_results: Vec::new(),
        }
    }

    pub fn add_search_result(&mut self, url: &str, title: &str, extract: &str) {
        self.search_results
            .push(SearchResult::new(url, title, extract));
    }

    pub fn len(&self) -> usize {
        self.search_results.len()
    }
}


fn get_query_regex(query: &str) -> regex::Regex {
    let query = "\\b".to_owned() + query
        .split_whitespace()
        .map(|word| regex::escape(word))
        .collect::<Vec<String>>()
        .join("\\b|\\b").as_str() + "\\b";
    regex::Regex::new(&query).unwrap()
}

fn get_features(query_tokens: Vec<&str>, search_result: SearchResult) -> HashMap<&str, f64> {
    let mut features = HashMap::new();

    let parsed_url =
        url::Url::parse(&search_result.url).unwrap_or(Url::parse(MISSING_URL).unwrap());
    let domain = parsed_url.domain().unwrap_or("");
    let path = parsed_url.path();

    for (part, name, is_url) in [
        (search_result.title.as_str(), "title", false),
        (search_result.extract.as_str(), "extract", false),
        (domain, "domain", true),
        (path, "path", true),
    ] {
        features.insert(name, 1.0);
    }

    features
}

#[cfg(test)]
mod tests {
    #[test]
    fn construct_some_search_results() {
        let mut ranker = super::Ranker::new("url");
        ranker.add_search_result("https://en.wikipedia.org/wiki/URL", "URL", "A URL is a reference to a web resource that specifies its location on a computer network and a mechanism for retrieving it.");

        assert_eq!(ranker.len(), 1);
    }

    #[test]
    fn test_get_query_regex() {
        let query = "web page";
        let regex = super::get_query_regex(query);
        assert_eq!(regex.as_str(), "\\bweb\\b|\\bpage\\b");
    }
}
