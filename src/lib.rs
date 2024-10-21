mod utils;

use std::i8::MAX;
use arrayvec::ArrayString;
use wasm_bindgen::prelude::*;

const MAX_URL_LENGTH: usize = 150;
const MAX_TITLE_LENGTH: usize = 65;
const MAX_EXTRACT_LENGTH: usize = 155;

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
            url: ArrayString::from(&url[..MAX_URL_LENGTH]).unwrap(),
            title: ArrayString::from(&title[..MAX_TITLE_LENGTH]).unwrap(),
            extract: ArrayString::from(&extract[..MAX_EXTRACT_LENGTH]).unwrap(),
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
        self.search_results.push(SearchResult::new(url, title, extract));
    }


}