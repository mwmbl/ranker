import * as wasm from "ranker";



// Get the search input
let searchInput = document.getElementById("search");
let searchButton = document.getElementById("search-button");

// Query the Mwmbl API at https://api.mwmbl.org/api/v1/search/raw?s=SEARCH_TERM
// then re-rank using the Ranker
searchButton.addEventListener("click", async (e) => {
  let searchTerm = searchInput.value;
  let ranker = wasm.Ranker.new(searchTerm);
  let response = await fetch(`https://api.mwmbl.org/api/v1/search/raw?s=${searchTerm}`);
  let data = await response.json();
  console.log("Data", data);
  data.results.forEach((result) => {
    // Check for nulls
    if (result.url == null) {
      result.url = "";
    }
    if (result.title == null) {
      result.title = "";
    }
    if (result.extract == null) {
      result.extract = "";
    }
    ranker.add_search_result(result.url, result.title, result.extract);
  });
  let rankedData = ranker.rank();
  console.log(rankedData);
});

