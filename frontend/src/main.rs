use dioxus::{html::{img::src, img::alt}, prelude::*};
use reqwest::Client;
use serde::{Deserialize, Serialize};


const MAIN_CSS: Asset = asset!("/assets/main.css");
const SEARCH_ICON: Asset = asset!("/assets/search-icon.png");
pub fn main() {
    dioxus::launch(App);
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Anime {
    title: String,
    picture: Option<String>,
}

#[component]
pub fn App() -> Element {
    let mut search_input = use_signal(|| "".to_string());
    let mut submitted_title = use_signal(|| String::new());

    let mut search_results: Signal<Vec<Anime>> = use_signal(|| vec![]);

    use_effect(move || {
        let query = search_input.read().clone();
        let mut results = search_results.clone();
        spawn(async move {
            if query.is_empty() {
                results.set(vec![]);
                return;
            }

            let client = Client::new();
            if let Ok(res) = client
                .get(format!("http://localhost:3000/search?query={}", query))
                .send()
                .await
            {
                if let Ok(names) = res.json::<Vec<Anime>>().await {
                    results.set(names);
                    
                }
            }
        });
        ()
    });
// add reactive drop shadow that increases on hover
// if user types somethins and then removes it the search results default to printing all the shows
// change to check names and synonyms together and ignore if it already exists
// change results to show up in bigger grids 
    rsx! {
        document::Link{rel: "stylesheet", href: MAIN_CSS}
        div {
            id:"top_div",
            h1 { "My Watch List" }
            div {
                id:"input_div",
                input {
                    id: "Search_Bar",
                    type: "text",
                    value: "{search_input}",
                    oninput: move |event| {
                        search_input.set(event.value());
                    },
                    onkeydown: move |event| {
                        if event.code().to_string() == "ENTER".to_string() {
                            submitted_title.set(search_input.read().clone()); }
                    }
                }
            button {
                id: "Search_button",
                onclick: move |_| {
                    submitted_title.set(search_input.read().clone());
                },
                img { 
                    id: "Search_Icon",
                    src: "{SEARCH_ICON}",
                    alt: "search",
                }
            } 
        }

    }
    if !search_results.read().is_empty() {
        div {  
            class: "dropdown",
            for anime in search_results.read().iter() {
                div { 
                    class: "dropdown_items", 
                img {  
                    class: "dropdown_images",
                    src: anime.picture.clone().unwrap_or("{SEARCH_ICON}".to_string()),
                    alt: "thumbanil"
                }
                span {  
                    class: "span_items",
                    "{anime.title}"
                } }
            }
        } 

    }
}
}
