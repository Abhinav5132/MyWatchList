
use crate::frontend::*;
use dioxus::desktop::{use_window};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use login_popup::Login;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Anime {
    pub id: i64,
    pub title: String,
    pub picture: Option<String>,
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
pub fn trending_component() -> Element{
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

#[component]
pub fn Searchpg() -> Element {
    let mut show_login = use_signal(|| false);
    let mut search_input = use_signal(|| "".to_string());
    let mut submitted_title = use_signal(|| String::new());
    let mut fade_direction = use_signal(|| "fade-in");
    let navigator = use_navigator();
    let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .expect("Failed to build client");
            provide_context(client);
    let search_results: Signal<Vec<Anime>> = use_signal(|| vec![]);
    let mut page: Signal<i32> = use_signal(|| 1);
    let client = use_context::<Client>();

    use_effect(move || {
        let query = search_input.read().clone();
        let page = page.read().clone();
        let mut results = search_results.clone();
        let client = client.clone();
        spawn(async move {
            if query.is_empty() {
                results.set(vec![]);
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            if let Ok(res) = client
                .get(format!(
                    "http://localhost:3000/search?query={}&page={}",
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
                        onclick: move |_| {
                                let navigator = navigator.clone();
                                navigator.push(crate::frontend::router::routes::ListsPgFn { user_id: *USERID.read() });
                            //should technically redirect to the page with a list of all playlists 
                            },
                        img {
                            class: "Feeling_icon",
                            src: "{PLAYLIST}",
                            alt:"Playlists",
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
                        onclick: move |_| {
                            fade_direction.set("fade-in");
                            show_login.set(true);
                        },
                        img {
                            class:"Feeling_icon",
                            src:"{NOPFP}",
                            
                        }
                        
                    }
                }
            }

            if *show_login.read(){
                div { 
                    class:"modal_overlay_search {fade_direction}",
                    onclick: move |_| {
                        fade_direction.set("fade-out");
                        show_login.set(false)
                    },
                    div { 
                        class: "modal_container_search",
                        onclick: move |e| e.stop_propagation(),
                        Login { 
                            on_close: move || {
                                fade_direction.set("fade-out");
                                show_login.set(false)
                            }
                        }

                    }
                }
            }

            if search_results.read().is_empty(){
                trending_component {}
            }

            if !search_results.read().is_empty() {
                div {
                    class: "dropdown_search",
                    for anime in search_results.read().iter().cloned() {
                        div {
                            class: "dropdown_items_search",
                            onclick: move |_| {
                                navigator.push(crate::frontend::router::routes::Details { id: anime.id.clone() });
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
