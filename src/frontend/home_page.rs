
use crate::frontend::*;
use dioxus::desktop::{use_window};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Anime {
    pub id: i64,
    pub title: String,
    pub largeImage: Option<String>,
}

#[derive(Serialize,Deserialize, Clone)]
struct ScrollingResults{
    id: i64,
    title_english: String,
    title_romanji: String,
    banner_image: String,
    averageScore: f32,
    description: String,
    duration: u32,
    format: String
}


#[derive(Serialize,Deserialize,Clone)]
struct TrendingResults{
    id: i64,
    title_english: String,
    title_romanji: String,
    thumbnail: String,
    averageScore: u32
}

#[derive(Serialize,Deserialize,Clone)]
struct TrendingResponse {
    new_popular: Vec<TrendingResults>,
    most_popular: Vec<TrendingResults>,
    scroll_popular: Vec<ScrollingResults>,
}

#[component]
pub fn HomePage() -> Element{
    let mut trending_results:Signal<TrendingResponse> = use_signal(|| TrendingResponse { new_popular: vec![], most_popular: vec![], scroll_popular: vec![] });
    let navigator = use_navigator();
    let client = use_context::<Client>();
    let mut current_index = use_signal(|| 0);

    // for trending 
    let mut top_index = use_signal(|| 0);
    
    // for new_trending 
    let mut top_index_new = use_signal(|| 0);

    let window = use_window();
    let page_size = use_signal(|| 1 as usize);

    use_future(move || {
        let window = window.clone();
        let mut page_size = page_size.clone();
        async move {
            loop {
            
            let width = (window.inner_size().width as usize / 205).max(1);
            page_size.set(width);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
 
        }}});   


    use_effect(move || {
        let client = client.clone();
        
        spawn(async move {
            if let Ok(res) = client.get(
                format!("http://localhost:3000/trending")
            ).send().await {
                if let Ok(names) = res.json::<TrendingResponse>().await{
                    trending_results.set(names)
                }
            }

        });
        }
    );

    use_future(move || {
        let trending_results = trending_results.clone();
        let mut current_index = current_index.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                let trending = trending_results.read();
                let len = trending.scroll_popular.len();
                if len > 0 {
                    let next = (*current_index.read() + 1) % len;
                    current_index.set(next);
                }
            }
        }
    });
    let trending = trending_results.read().clone();
    // scrolling results
    let len = trending.scroll_popular.len();
    let current = {
        if len == 0{
            None
        } else {
            trending.scroll_popular.get(*current_index.read()).cloned()
        }
    };

    let most_popular = trending.most_popular.clone();
    let length  = most_popular.len();
    let start = *top_index.read();
    let end = (start + *page_size.read()).min(length);
    let current_anime = most_popular[start..end].to_vec();

    let new_popular = trending.new_popular.clone();
    let length_new_popular = new_popular.len();
    let start_new_popular = *top_index_new.read();
    let end_new = (start_new_popular + *page_size.read()).min(length_new_popular);
    let current_anime_new = new_popular[start_new_popular..end_new].to_vec();

    rsx!(
        div { 
            id:"Scrolling_suggestion_search",
            if let Some(current_anime) = current {
            div {
                class:"scroll_item_wrapper",
                img {
                    class: "Scrollable_images_search",
                    onclick: move |_| {
                    let navigator = navigator.clone();
                    navigator.push(crate::frontend::router::routes::Details { id: current_anime.id }); },

                    src:format!("{}", current_anime.banner_image),
                    alt: "Trending anime",
                    
                },
                
                div { 
                    id:"Scrolling_description_search",
                    onclick: move |_| {
                    let navigator = navigator.clone();
                    navigator.push(crate::frontend::router::routes::Details { id: current_anime.id }); },
                    h2 { "{current_anime.title_romanji} / {current_anime.title_english}" },
                    p {
                        id:"Scrolling_description",
                        "{current_anime.description}"  
                    }
                    div { 
                        id:"Scrolling_details_search",
                        h4 { "Score: {current_anime.averageScore}" },
                        h4 { "Duration: {current_anime.duration}" },
                        h4 { "Format: {current_anime.format}" },
                    }
                }

                

                div {
                    id:"Scrolling_buttons_div",
                    button { 
                        id:"Scrolling_button_prev",
                        onclick: move |_| {
                            let mut index = *current_index.read();
                            if index == 0 {
                                index = len - 1; 
                            } else {
                                index -= 1;
                            }
                            current_index.set(index);
                        },
                        img {
                            src:"{PREV}"
                        },
                    },
                    button { 
                        id:"Scrolling_button_next",
                        onclick: move |_| {
                            let next = (*current_index.read() + 1) % len;
                            current_index.set(next);
                        },
                        img {
                            src:"{NEXT}"
                        },
                    
                    } 
                    
                }
            } 
        }   
    }

        div {
            id: "Top_trending_div_container",
            h2 { "Top Trending" },
            div { 
                id: "Top_trending_row",
                for trending_anime in current_anime{
                    div {
                        class: "Top_trending_div",
                        onclick: move |_| {
                                navigator.push(crate::frontend::router::routes::Details { id: trending_anime.id });
                            },
                        img { 
                            class: "trending_thumbnail",
                            src:"{trending_anime.thumbnail}"
                        },
                        h5 { "{trending_anime.title_romanji}" }
                    }
                } 
            }
        }

        div {
                id: "Top_trending_buttons",
                button {
                    onclick: move |_| {
                        let mut index = *top_index.read();
                        if index >= *page_size.read() {
                            index -= *page_size.read();
                        } else {
                            index = length - 1; // wrap to last page
                        }
                        top_index.set(index);
                    },
                    "Prev"
                }
                button {
                    onclick: move |_| {
                        let mut index = *top_index.read();
                        if index + *page_size.read() < length{
                            index += *page_size.read();
                        } else {
                            index = 0;
                        }
                        top_index.set(index);
                    },
                    "Next"
                }
            }
        
        div {
            id: "Top_trending_div_container",
            h2 { "New & Trending" },
            div { 
                id: "Top_trending_row",
                for trending_anime in current_anime_new{
                    div {
                        class: "Top_trending_div",
                        onclick: move |_| {
                                navigator.push(crate::frontend::router::routes::Details { id: trending_anime.id });
                        },
                        img { 
                            class: "trending_thumbnail",
                            src:"{trending_anime.thumbnail}"
                        },
                        h5 { "{trending_anime.title_romanji}" }
                    }
                } 
            }
        }

        div {
                id: "Top_trending_buttons",
                button {
                    onclick: move |_| {
                        let mut index = *top_index_new.read();
                        if index >= *page_size.read() {
                            index -= *page_size.read();
                        } else {
                            index = length - 1; // wrap to last page
                        }
                        top_index_new.set(index);
                    },
                    "Prev"
                }
                button {
                    onclick: move |_| {
                        let mut index = *top_index_new.read();
                        if index + *page_size.read() < length{
                            index += *page_size.read();
                        } else {
                            index = 0;
                        }
                        top_index_new.set(index);
                    },
                    "Next"
                }
            }
    )
}

