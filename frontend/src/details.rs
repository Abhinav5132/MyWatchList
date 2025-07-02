use std::result;

use crate::*;
use dioxus::html::audio::src;
use reqwest::Client;
use serde::{Serialize, Deserialize};
const DETAILS_CSS: Asset = asset!("/assets/details_page.css");

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct FullAnimeResult {
    title: String,
    format: String,
    episodes: i32,
    status:String,
    anime_season: String,
    anime_year: i32,
    picture: String,
    duration: i32,
    score: f32,
    studio: Option<Vec<String>>,
    synonyms: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}

#[component]
pub fn Details(id: i32) -> Element{
    let mut anime_details: Signal<Option<FullAnimeResult>> = use_signal(|| None);
    let navigator = use_navigator();
    use_effect(move ||{
        let mut details = anime_details.clone();
        spawn(async move {
            let client = Client::new();
            if let Ok(res) = client.get(format!("http://localhost:3000/details?query={}", id))
            .send().await{
                if let Ok(detail) = res.json::<FullAnimeResult>().await{
                    details.set(Some(detail));
                }
            }
        });
        ()
    });
    
    rsx!{
    match anime_details.read().as_ref(){
        Some(details) => rsx!{
            document::Link{rel: "stylesheet", href: DETAILS_CSS},
            div{
                id:"Title_div",
            h3 { "{ details.title }" },
            }
            div {  
                id:"picture_div",
                img {
                    id:"Detail_image",
                    src: "{ details.picture }",
                    alt: "picture"
                    }
                }
                


                button {  
                    onclick: move |_| {
                        navigator.push(crate::router::routes::Searchpg {  });
                    }
                }

            },
        
        None => rsx!{
            h1 { "Loading" }
        }
    }
}
}