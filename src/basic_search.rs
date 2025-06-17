use crate::global_db::get_db_pool;
use dioxus::prelude::*;
use serde_json::to_string;

const MAIN_CSS: Asset = asset!("/assets/main.css");

pub fn search_main() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    let conn = get_db_pool();

    struct Anime {
        title: String,
    }
    let mut search_input = use_signal(|| "".to_string());
    let mut submitted_title = use_signal(|| "Please enter anime name here".to_string());
    rsx! {
        div {
            document::Link{rel: "stylesheet", href: MAIN_CSS}
            h1 { "Search for an anime" }
            div {
                input {
                    id: "Search_Bar",
                    type: "search",
                    value: "{search_input}",
                    oninput: move |event| {
                        search_input.set(event.value());
                    },
                    onkeydown: move |event| {
                        if event.code().to_string() == "ENTER".to_string() {
                            submitted_title.set(search_input.read().clone());
                        }
                    }
                }
            button {
                id: "Search_buttone",
                onclick: move |_| {
                    submitted_title.set(search_input.read().clone());
                },
                "search"
                }
            }

        }

    }
}

