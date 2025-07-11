use dioxus::{html::{img::src, img::alt}, prelude::*};
use dioxus_router::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const MAIN_CSS: Asset = asset!("/assets/search_page.css");
const SEARCH_ICON: Asset = asset!("/assets/search-icon.png");
const PREV: Asset = asset!("/assets/prev-page.png");
const NEXT: Asset = asset!("/assets/next-page.png");

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Anime {
    id: i32,
    title: String,
    picture: Option<String>,
}

#[component]
pub fn Searchpg() -> Element {
    let mut search_input = use_signal(|| "".to_string());
    let mut submitted_title = use_signal(|| String::new());
    let navigator = use_navigator();
    let mut search_results: Signal<Vec<Anime>> = use_signal(|| vec![]);
    let mut page:Signal<i32> = use_signal(|| 1);
    use_effect(move || {
        let query = search_input.read().clone();
        let page = page.read().clone();
        let mut results = search_results.clone();
        spawn(async move {
            if query.is_empty() {
                results.set(vec![]);
                return;
            }
            let client = Client::new();
            if let Ok(res) = client
                .get(format!("http://localhost:3000/search?query={}&page={}", query, page))
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
                    submitted_title.set(search_input.read().clone()) // for now this button does jack shit
                    
                    ;
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
            for anime in search_results.read().iter().cloned() {
                div { 
                    class: "dropdown_items", 
                    onclick: move |_| {
                        navigator.push(crate::router::routes::Details { id: anime.id.clone() });
                    },
                img {  
                    class: "dropdown_images",
                    loading: "lazy",
                    src: anime.picture.clone().unwrap_or("{SEARCH_ICON}".to_string()),
                    alt: "thumbanil"
                }
                span {  
                    class: "span_items",
                    "{anime.title}"
                } }
            }
    }
    div {  
            button {  
                onclick: move |_| {
                    page.with_mut(|p| {
                    *p = (*p - 1).max(1);
                    });
                    },
                    img { 
                        src: "{PREV}",
                    }

            }

            button {  
                onclick: move |_| {
                    page.with_mut(|p| *p += 1);
                    },
                    img { 
                        src: "{NEXT}",
                    }

            }
        } 
    }
}
}
