use crate::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
const DETAILS_CSS: Asset = asset!("/assets/details_page.css");
const HEART:Asset = asset!("/assets/heart.png");
const TRAHSH:Asset = asset!("/assets/bin.png");
const TICK:Asset = asset!("/assets/check-mark.png");
const NOPFP:Asset = asset!("/assets/No_pfp.jpg");

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct FullAnimeResult {
    title: String,
    format: String,
    description: String,
    episodes: i32,
    status: String,
    anime_season: String,
    anime_year: i32,
    picture: String,
    duration: i32,
    score: f32,
    trailer_url: String,
    studio: Option<Vec<String>>,
    synonyms: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}

#[component]
pub fn Details(id: i32) -> Element {
    let mut anime_details: Signal<Option<FullAnimeResult>> = use_signal(|| None);
    let navigator = use_navigator();
    use_effect(move || {
        let mut details = anime_details.clone();
        spawn(async move {
            let client = Client::new();
            if let Ok(res) = client
                .get(format!("http://localhost:3000/details?query={}", id))
                .send()
                .await
            {
                if let Ok(detail) = res.json::<FullAnimeResult>().await {
                    details.set(Some(detail));
                }
            }
        });
        ()
    });

    rsx! {
        match anime_details.read().as_ref(){
            Some(details) => {
            let hours = details.duration / 60;
            let minutes = details.duration % 60;
            let length = format!("{:02}:{:02}", hours, minutes);

            let rating = details.score / 10.0;
            rsx!{
                document::Link{rel: "stylesheet", href: DETAILS_CSS},
                div{
                    id:"Title_div",
                h3 { id: "Title",
                    "{ details.title }" },
                }

                div{
                    id: "top_div",

                    div {
                        id:"picture_div",
                        img {
                            id:"Detail_image",
                            src: "{ details.picture }",
                            alt: "picture"
                            }
                    }
                    div{
                        id:"Details_div",
                        div{
                            id: "format_details_div",
                            if details.format == "MOVIE" {

                                h4 {
                                    "Format: Movie"
                                }
                                h4 {
                                    "Duration: {length}"
                                }

                                h4 {
                                    "Status: {details.status}"
                                }

                                h4 {
                                    "Rating: {rating}"
                                }
                            }
                            else {
                                h4 { 
                                    "Format: {details.format}" 
                                }
                                h4 { 
                                    "Episodes : {details.episodes}" 
                                }
                                h4 {
                                    "Status: {details.status}"
                                }
                                h4 {
                                    "Rating: {rating}"
                                }
                            }
                        }
                        div {
                            id: "Description_div",
                            p {
                                "{details.description}"
                            }
                        }
                    }
                    div {
                        id: "Friends_div",
                        h3 { "Friends:" }
                        div {  
                            class: "Friend_card",
                            img { 
                                class: "PFP" ,
                                src: NOPFP
                            }
                            img { 
                                class: "Feeling_icon",
                                src: HEART 
                            }
                            h5 { "Diddyago liked this" }
                            
                        }
                        div {  
                            class: "Friend_card",
                            img { 
                                class: "PFP" ,
                                src: NOPFP
                            }
                            img { 
                                class: "Feeling_icon",
                                src: TRAHSH
                            }
                            h5 { "N hated this " }
                            
                        }
                        div {  
                            class: "Friend_card",
                            img { 
                                class: "PFP" ,
                                src: NOPFP
                            }
                            img { 
                                class: "Feeling_icon",
                                src: TICK 
                            }
                            h5 { "Diddyago Watched this" }
                            
                        }
                        div {  
                            class: "Friend_card",
                            img { 
                                class: "PFP" ,
                                src: NOPFP
                            }
                            img { 
                                class: "Feeling_icon",
                                src: HEART 
                            }
                            h5 { "Diddyago liked this" }
                            
                        }
                        div {  
                            class: "Friend_card",
                            img { 
                                class: "PFP" ,
                                src: NOPFP
                            }
                            img { 
                                class: "Feeling_icon",
                                src: HEART 
                            }
                            h5 { "Diddyago liked this" }
                            
                        }
                        button {
                            id:"show_more_friends_button",
                            "Show More" 
                        }
                    }
                }
        button {
            onclick: move |_| {
                navigator.push(crate::router::routes::Searchpg {  });
                }
            }
        }},


            None => rsx!{
                h1 { "Loading" }
            }
        }
    }
}
