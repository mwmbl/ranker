import * as wasm from "ranker";



// Get the search input
let searchInput = document.getElementById("search");
let searchButton = document.getElementById("search-button");

// Query the Mwmbl API at https://api.mwmbl.org/api/v1/search/raw?s=SEARCH_TERM
// then re-rank using the Ranker
searchButton.addEventListener("click", async (e) => {
  let searchTerm = searchInput.value;
  let ranker = wasm.Ranker.new(searchTerm);
  let terms = ranker.get_query_terms();
  console.log("Query terms", terms);
  for (const term of terms) {
    let response = await fetch(`https://api.mwmbl.org/api/v1/search/raw?s=${term}`);
    let data = await response.json();
    console.log("Data", data);
    for (const result of data.results) {
      ranker.add_search_result(result.url, result.title, result.extract);
    }
  }
  let rankedData = ranker.rank();
  console.log(rankedData);

  // Insert into the output div
  let outputDiv = document.getElementById("output");
  outputDiv.innerHTML = "";
  rankedData.forEach((result) => {
    let div = document.createElement("div");
    div.innerHTML = `
      <a href="${result.url}">${result.url}</a>
      <h3>${result.title}</h3>
      <p>${result.extract}</p>
      <br><br>
    `;
    outputDiv.appendChild(div);
  });
});

