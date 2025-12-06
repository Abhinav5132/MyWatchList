use reqwest::Client;
use serde_json::json;
use crate::frontend::{home_page::Anime, logedin_dropdown::loged_in_dropdown, login_popup::Login};
pub use crate::frontend::*;


#[derive(Serialize, Deserialize)]
pub struct UserDetails{
    pub username: String,
    pub user_email: String,
    pub user_pfp: String,
}

#[component]
pub fn TitleBar() -> Element {
    let mut show_login = use_signal(|| false);
    let mut search_input = use_signal(|| "".to_string());
    let mut submitted_title = use_signal(|| String::new());
    let mut fade_direction = use_signal(|| "fade-in");
    let mut refetch_signal = use_signal(|| false);

    provide_context(refetch_signal);
    let navigator = use_navigator();
    let client = Client::new();
    provide_context(client);
    let search_results: Signal<Vec<Anime>> = use_signal(|| vec![]);
    let mut page: Signal<i32> = use_signal(|| 1);
    let client = use_context::<Client>();

    let mut username = use_signal(|| "".to_string());
    let mut user_email = use_signal(|| "".to_string());
    let mut user_pfp = use_signal(|| "".to_string());

    let mut show_logedin_dropdown = use_signal(|| false);

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

                
    use_effect(move || {
        let _ = refetch_signal.read();
        let user_name = USERNAME.cloned();
        let user_id = USERID.cloned();
        if user_id != -1 && user_name != "".to_string(){ 
            spawn (async move{
                let client = Client::new();
                if let Ok(res) = client.post("http://localhost:3000/get_user_details").json(&json!({ // can posibly turned into a macro
                    "user_id": user_id,
                })).send().await {
                    if let Ok(usr_dets) = res.json::<UserDetails>().await{
                        username.set(usr_dets.username);
                        user_email.set(usr_dets.user_email);
                        user_pfp.set(usr_dets.user_pfp);
                    }
                }
            });
        }}
    );
    
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
                        onclick: move |_| {
                            let navigator = navigator.clone();
                            navigator.push(crate::frontend::router::routes::HomePage {  });
                        },
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
                                navigator.push(crate::frontend::router::routes::FriendPage {  });
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
                            submitted_title.set(search_input.read().clone()); } // change fade direction here as welle
                        }
                    }
                    // change to no pfp if not loged in, if loged in change to the users pfp
                    if !(*USERID.read() != -1 && *USERNAME.read() != "".to_string()){
                        button {
                            class:"Icon_but = tryton_search",
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
                    } else {
                        button {
                            class:"Icon_button_search",
                            id:"Account_button_search",
                            onclick: move |e|  {
                                e.stop_propagation();
                                show_logedin_dropdown.set(true);
                                
                            },
                            img {
                                class:"Feeling_icon",
                                src:"{user_pfp}",
                                
                            }
                            
                        }

                        if *show_logedin_dropdown.read(){
                            loged_in_dropdown {
                                username: username.read(), 
                                user_email: user_email.read(),
                                user_image: user_pfp.read(),
                                onclose: move |_| {
                                    show_logedin_dropdown.set(false);
                                }
                            }
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
                                src: anime.largeImage.clone().unwrap_or("{SEARCH_ICON}".to_string()),
                                alt: "thumbanil"
                            }
                            span {
                                class: "span_items_search",
                                "{anime.title}"
                            } 
                        }
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
