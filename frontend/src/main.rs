use dioxus::{html::{img::src, img::alt}, prelude::*};
use reqwest::Client;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const SEARCH_ICON: Asset = asset!("/assets/search-icon.png");
pub fn main() {
    dioxus::launch(App);
}
#[derive(Clone)]
struct Anime {
    title: String,
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
                if let Ok(names) = res.json::<Vec<String>>().await {
                    let anime_list = names.into_iter().map(|title| Anime { title }).collect();
                        results.set(anime_list);
                    
                }
            }
        });
        ()
    });
// add reactive drop shadow that increases on hover
// if user types somethins and then removes it the search results default to printing all the shows
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
                id: "Search_buttone",
                onclick: move |_| {
                    submitted_title.set(search_input.read().clone());
                },
                img { 
                    id: "Search_Icon",
                    src: "{SEARCH_ICON}",
                    alt: "search",
                 }
                } 
            ul {
                id: "Result_list",
                for anime in search_results.read().iter(){
                    li {id: "List_Item","{anime.title}"}
                }
            }

        }

    }
}
}