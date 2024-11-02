mod utils;

use arrayvec::ArrayString;
use regex::Regex;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use url::Url;
use wasm_bindgen::prelude::*;

const MAX_URL_LENGTH: usize = 150;
const MAX_TITLE_LENGTH: usize = 65;
const MAX_EXTRACT_LENGTH: usize = 155;
const MATCH_EXPONENT: f64 = 2.0;

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

#[derive(Default, Debug)]
struct MatchFeatures {
    last_char: u8,
    length: u8,
    total_possible_length: u8,
    num_terms: u8,
    score: f32,
    term_proportion: f32,
}

#[derive(Default, Debug)]
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
    num_unique_terms: u8,
    query_regex: Regex,
    search_results: Vec<SearchResult>,
}

#[wasm_bindgen]
impl Ranker {
    pub fn new(query: &str) -> Ranker {
        let (query_regex, num_unique_terms, total_possible_match_length) = get_query_regex(query);
        Ranker {
            query: query.to_string(),
            total_possible_match_length,
            num_unique_terms,
            query_regex,
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

fn get_query_regex(query: &str) -> (Regex, u8, u8) {
    let unique_query_terms = query
        .split_whitespace()
        .map(|word| regex::escape(word))
        .collect::<HashSet<String>>();
    let query = "\\b".to_owned()
        + unique_query_terms
            .clone()
            .into_iter()
            .collect::<Vec<String>>()
            .join("\\b|\\b")
            .as_str()
        + "\\b";
    let term_length_sum: usize = unique_query_terms.iter().map(|term| term.len()).sum();
    let term_length_sum = u8::try_from(term_length_sum).unwrap_or(u8::MAX);
    let num_unique_terms = u8::try_from(unique_query_terms.len()).unwrap_or(u8::MAX);
    (Regex::new(&query).unwrap(), num_unique_terms, term_length_sum)
}

fn score_result(
    query_regex: Regex,
    search_result: SearchResult,
    total_possible_length: u8,
    num_unique_terms: u8,
) -> f32 {
    let features = get_features(query_regex, search_result, total_possible_length, num_unique_terms);
    let length_penalty = f32::exp(-0.04 * search_result.url.len() as f32);
    let match_score = (4.0 * features.title_match.score
        + features.extract_match.score
        + 4.0 * features.domain_match.score // TODO: use tokenized domain match as well
        + 2.0 * features.path_match.score);

    // TODO: check the minimum number of terms matching
    // TODO: get domain score

    match_score * length_penalty / 10.0
}


fn get_features(
    query_regex: Regex,
    search_result: SearchResult,
    total_possible_length: u8,
    num_unique_terms: u8,
) -> Features {
    let parsed_url =
        url::Url::parse(&search_result.url).unwrap_or(Url::parse(MISSING_URL).unwrap());
    let domain = parsed_url.domain().unwrap_or("");
    let path = parsed_url.path();

    let mut features = Features::default();
    for (i, (part, name, is_url)) in [
        (search_result.title.as_str(), "title", false),
        (search_result.extract.as_str(), "extract", false),
        (domain, "domain", true),
        (path, "path", true),
    ]
    .iter()
    .enumerate()
    {
        let part_lower = part.to_lowercase();
        let matches = query_regex.find_iter(part_lower.as_str());
        let mut last_match_char = 1;
        let mut seen_terms = HashSet::new();
        let mut match_length = 0;
        // println!("Num matches for {}: {}", name, matches.count());
        println!("Query regex: {:?}", query_regex);
        println!("Part: {:?}", part);
        for m in matches {
            let match_term = m.as_str();
            println!("Name {:?} Match: {:?}", name, match_term);
            if seen_terms.contains(match_term) {
                continue;
            }
            seen_terms.insert(match_term);
            last_match_char = m.end();
            match_length += m.end() - m.start();
        }

        let match_length = u8::try_from(match_length).unwrap_or(u8::MAX);
        let last_match_char = u8::try_from(last_match_char).unwrap_or(u8::MAX);
        let num_terms = u8::try_from(seen_terms.len()).unwrap_or(u8::MAX);

        let score = f64::powf(
            MATCH_EXPONENT,
            match_length as f64 - total_possible_length as f64,
        ) / last_match_char as f64;
        let score = score as f32;

        let match_features = MatchFeatures {
            last_char: last_match_char,
            length: match_length,
            total_possible_length,
            num_terms,
            score,
            term_proportion: num_terms as f32 / num_unique_terms as f32,
        };
        if (*name).eq("title") {
            features.title_match = match_features;
        } else if (*name).eq("extract") {
            features.extract_match = match_features;
        } else if (*name).eq("domain") {
            features.domain_match = match_features;
        } else if (*name).eq("path") {
            features.path_match = match_features;
        } else {
            panic!("Unknown part: {}", name);
        }
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
        let query = "web web";
        let (regex, num_unique_terms, max_length) = super::get_query_regex(query);
        assert_eq!(regex.as_str(), "\\bweb\\b");
        assert_eq!(max_length, 3);
        assert_eq!(num_unique_terms, 1);
    }

    #[test]
    fn test_get_features() {
        let query = "url";
        let (regex, num_unique_terms, total_possible_length) = super::get_query_regex(query);
        let search_result = super::SearchResult::new("https://en.wikipedia.org/wiki/URL", " URL", "A URL is a reference to a web resource that specifies its location on a computer network and a mechanism for retrieving it.");
        let features = super::get_features(regex, search_result, total_possible_length, num_unique_terms);
        println!("{:#?}", features);
        assert_eq!(features.title_match.length, 3);
        assert_eq!(features.title_match.last_char, 4);
        assert_eq!(features.title_match.num_terms, 1);
        assert_eq!(features.title_match.score, 0.25);
        assert_eq!(features.title_match.term_proportion, 1.0);
    }
}
