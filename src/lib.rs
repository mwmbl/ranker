mod utils;

use arrayvec::ArrayString;
use regex::Regex;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashSet;
use std::convert::TryFrom;
use serde::ser::SerializeStruct;
use url::Url;
use wasm_bindgen::prelude::*;

const MAX_URL_LENGTH: usize = 200;
const MAX_TITLE_LENGTH: usize = 100;
const MAX_EXTRACT_LENGTH: usize = 200;
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

fn shorten_string(s: &str, max_length: usize) -> &str {
    // Shorten the string but check that we don't slice within a character
    if s.len() <= max_length {
        return s;
    }

    let mut end = max_length;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[derive(Clone, Copy, Debug)]
struct SearchResult {
    pub url: ArrayString<MAX_URL_LENGTH>,
    pub title: ArrayString<MAX_TITLE_LENGTH>,
    pub extract: ArrayString<MAX_EXTRACT_LENGTH>,
}

impl Serialize for SearchResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SearchResult", 3)?;

        state.serialize_field("url", &self.url.as_str())?;
        state.serialize_field("title", &self.title.as_str())?;
        state.serialize_field("extract", &self.extract.as_str())?;
        state.end()
    }
}



impl SearchResult {
    pub fn new(url: &str, title: &str, extract: &str) -> SearchResult {
        console_error_panic_hook::set_once();
        SearchResult {
            url: ArrayString::from(shorten_string(url, MAX_URL_LENGTH)).unwrap(),
            title: ArrayString::from(shorten_string(title, MAX_TITLE_LENGTH)).unwrap(),
            extract: ArrayString::from(shorten_string(extract, MAX_EXTRACT_LENGTH)).unwrap(),
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

    pub fn get_query_terms(&self) -> JsValue {
        let tokens = self.query.split_whitespace().collect::<Vec<&str>>();
        let bigrams = tokens.windows(2).map(|pair| pair.join(" ")).collect::<Vec<String>>();
        let unique_tokens = tokens.iter().map(|s| s.to_string()).collect::<HashSet<String>>();
        let unique_bigrams = bigrams.iter().collect::<HashSet<&String>>();
        let mut terms = unique_tokens.iter().collect::<Vec<&String>>();
        terms.extend(unique_bigrams.iter());
        serde_wasm_bindgen::to_value(&terms).unwrap()
    }

    pub fn add_search_result(&mut self, url: &str, title: &str, extract: &str) {
        self.search_results
            .push(SearchResult::new(url, title, extract));
    }

    pub fn len(&self) -> usize {
        self.search_results.len()
    }

    // Return the index of each search result in the order of the rank
    pub fn rank(&self) -> JsValue {
        let mut scored_results = self
            .search_results
            .iter()
            .map(|result| {
                (
                    result,
                    score_result(
                        self.query_regex.clone(),
                        *result,
                        self.total_possible_match_length,
                        self.num_unique_terms,
                    ),
                )
            })
            .collect::<Vec<(&SearchResult, f32)>>();
        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let ranked_results: Vec<&SearchResult> = scored_results.iter().map(|(i, _)| i.clone()).collect();
        serde_wasm_bindgen::to_value(&ranked_results).unwrap()
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
    (
        Regex::new(&query).unwrap(),
        num_unique_terms,
        term_length_sum,
    )
}

fn score_result(
    query_regex: Regex,
    search_result: SearchResult,
    total_possible_length: u8,
    num_unique_terms: u8,
) -> f32 {
    let features = get_features(
        query_regex,
        search_result,
        total_possible_length,
        num_unique_terms,
    );
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
        let features = super::get_features(
            regex,
            search_result,
            total_possible_length,
            num_unique_terms,
        );
        println!("{:#?}", features);
        assert_eq!(features.title_match.length, 3);
        assert_eq!(features.title_match.last_char, 4);
        assert_eq!(features.title_match.num_terms, 1);
        assert_eq!(features.title_match.score, 0.25);
        assert_eq!(features.title_match.term_proportion, 1.0);
    }
}
