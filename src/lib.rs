mod utils;

use arrayvec::ArrayString;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use regex::Regex;
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


struct MatchFeatures {
    last_char: u8,
    length: u8,
    total_possible_length: u8,
    num_terms: u8,
    score: f32,
    term_proportion: f32,
}

struct Features {
    title_match: MatchFeatures,
    extract_match: MatchFeatures,
    domain_match: MatchFeatures,
    path_match: MatchFeatures,
}


#[wasm_bindgen]
struct Ranker {
    query: String,
    total_possible_match_length: u8,
    query_regex: Regex,
    search_results: Vec<SearchResult>,
}

#[wasm_bindgen]
impl Ranker {
    pub fn new(query: &str) -> Ranker {
        let (query_regex, total_possible_match_length) = get_query_regex(query);
        Ranker {
            query: query.to_string(),
            query_regex,
            total_possible_match_length,
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


fn get_query_regex(query: &str) -> (Regex, u8) {
    let unique_query_terms = query
        .split_whitespace()
        .map(|word| regex::escape(word))
        .collect::<HashSet<String>>();
    let query = "\\b".to_owned() + unique_query_terms.clone().into_iter()
        .collect::<Vec<String>>()
        .join("\\b|\\b").as_str() + "\\b";
    let term_length_sum: usize = unique_query_terms.iter().map(|term| term.len()).sum();
    let term_length_sum = u8::try_from(term_length_sum).unwrap_or(u8::MAX);
    (Regex::new(&query).unwrap(), term_length_sum)
}

fn get_features(query_regex: Regex, search_result: SearchResult) -> Features {
    let parsed_url =
        url::Url::parse(&search_result.url).unwrap_or(Url::parse(MISSING_URL).unwrap());
    let domain = parsed_url.domain().unwrap_or("");
    let path = parsed_url.path();

    let match_features: [MatchFeatures; 4];
    for (i, (part, name, is_url)) in [
        (search_result.title.as_str(), "title", false),
        (search_result.extract.as_str(), "extract", false),
        (domain, "domain", true),
        (path, "path", true),
    ].iter().enumerate() {
        let matches = query_regex.find_iter(part);
        let mut last_match_char = 1;
        let mut seen_terms = HashSet::new();
        let mut match_length = 0;
        for m in matches {
            let match_term = m.as_str();
            if seen_terms.contains(match_term) {
                continue;
            }
            seen_terms.insert(match_term);
            last_match_char = m.end() as u8;
            match_length += m.end() - m.start();
        }
    }

    Features {
        title_match: MatchFeatures {
            last_char: 0,
            length: 0,
            total_possible_length: 0,
            num_terms: 0,
            score: 0.0,
            term_proportion: 0.0,
        },
        extract_match: MatchFeatures {
            last_char: 0,
            length: 0,
            total_possible_length: 0,
            num_terms: 0,
            score: 0.0,
            term_proportion: 0.0,
        },
        domain_match: MatchFeatures {
            last_char: 0,
            length: 0,
            total_possible_length: 0,
            num_terms: 0,
            score: 0.0,
            term_proportion: 0.0,
        },
        path_match: MatchFeatures {
            last_char: 0,
            length: 0,
            total_possible_length: 0,
            num_terms: 0,
            score: 0.0,
            term_proportion: 0.0,
        },
    }
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
        let query = "web web";
        let (regex, max_length) = super::get_query_regex(query);
        assert_eq!(regex.as_str(), "\\bweb\\b");
        assert_eq!(max_length, 3);
    }
}
