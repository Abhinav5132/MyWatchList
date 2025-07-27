use crate::*;
use dioxus::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
    let mut page: Signal<i32> = use_signal(|| 1);
    use_effect(move || {
        let query = search_input.read().clone();
        let page = page.read().clone();
        let mut results = search_results.clone();
        spawn(async move {
            if query.is_empty() {
                results.set(vec![]);
                return;
            }
            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap();
            if let Ok(res) = client
                .get(format!(
                    "https://localhost:3000/search?query={}&page={}",
                    query, page
                ))
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
            /*
            div {
                id:"top_div_search",
                h1 {
                    id: "h1_search",
                    "My Watch List"
                }
                div {
                    id:"input_div_search",
                    input {
                        id: "Search_Bar_search",
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
                        id: "Search_button_search",
                        onclick: move |_| {
                            submitted_title.set(search_input.read().clone()) // for now this button does jack shit
                            ;
                        },
                        img {
                            id: "Search_Icon_search",
                            src: "{SEARCH_ICON}",
                            alt: "search",
                        }
                    }
                }
            }*/
            body {
                id:"body_search",
            div {
                id:"header_div_search",
                div{
                    id:"header_div_left_search",
                    button {
                        class:"Icon_button_search",
                        id: "Menu_button_search",
                        onclick: move |_| {
                            //does noting for now
                        },
                        img {
                            class: "Feeling_icon",
                            src: "{MENU}",
                            alt: "MENU",
                        }
                    }
                    button{
                        class:"Icon_button_search",
                        id:"Home_buttons_search",
                        h1 {
                            id:"h1_search",
                            "MyWatchList",
                        }
                    }
                    button {
                        class:"Icon_button_search",
                        id:"Platlist_button_search",
                        img {
                            class: "Feeling_icon",
                            src: "{PLAYLIST}",
                            alt:"Playlists",
                            onclick: move |_| {
                            //does nothing for now redirect later
                            },
                       }
                    }

                    button {
                        class:"Icon_button_search",
                        id:"Freinds_button_search",
                        img {
                            class: "Feeling_icon",
                            src: "{FRIENDS}",
                            alt:"Playlists",
                            onclick: move |_| {
                            //does nothing for now redirect later
                            },
                        }
                    }
                }

                div {
                    id:"header_div_right_search",
                    input{
                    id: "Search_Bar_search",
                    type: "text",
                    value: "{search_input}", //background-image: url('searchicon.png');
                    placeholder:"Search..",
                    oninput: move |event| {
                        search_input.set(event.value());
                    },
                    onkeydown: move |event| {
                        if event.code().to_string() == "ENTER".to_string() {
                            submitted_title.set(search_input.read().clone()); }
                        }
                    }

                    button {
                        class:"Icon_button_search",
                        id:"Account_button_search",
                        img {
                            class:"Feeling_icon",
                            src:"{NOPFP}",
                            onclick: move |_| {
                            //does nothing for now redirect later
                            },
                        }
                    }
                }
            }

            if !search_results.read().is_empty() {
                div {
                    class: "dropdown_search",
                    for anime in search_results.read().iter().cloned() {
                        div {
                            class: "dropdown_items_search",
                            onclick: move |_| {
                                navigator.push(crate::router::routes::Details { id: anime.id.clone() });
                            },
                        img {
                            class: "dropdown_images_search",
                            loading: "eager",
                            src: anime.picture.clone().unwrap_or("{SEARCH_ICON}".to_string()),
                            alt: "thumbanil"
                        }
                        span {
                            class: "span_items_search",
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
}
