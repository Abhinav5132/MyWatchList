use reqwest::Client;
use crate::frontend::{home_page::Anime, login_popup::Login};
pub use crate::frontend::*;

#[component]
pub fn TitleBar() -> Element {
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
                div {
                    id: "page_content",
                    Outlet::<routes> { }
                }
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
