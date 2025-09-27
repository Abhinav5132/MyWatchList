use reqwest::ClientBuilder;
use crate::frontend::home_page::Anime;
pub use crate::frontend::*;

#[derive(Deserialize)]
pub struct AllAnimeSimple{
   pub anime: Vec<Anime>
}

#[derive(Serialize)]
struct FetchAnimes{
    watch_list_name: String,
    user_id: i64,
    page_no: i64,
}

#[component]
pub fn ListPgFn( list_name: String, user_id: i64) -> Element{
    let name = use_signal(|| list_name);
    let navigator = navigator();
    let mut page = use_signal(|| 1); // page starts at 1
    let mut anime = use_signal(|| vec![]);
    use_effect(move || {
        let page = page;
        spawn(async move {
            let client = ClientBuilder::new().danger_accept_invalid_certs(true).build().expect("failed to create client.");
            if let Ok(res) = client.get("http://localhost:3000/get-animes-from-list")
            .json(&FetchAnimes{
                watch_list_name: name.read().clone(),
                user_id: user_id.clone(),
                page_no: *page.read()
            }).send().await{
               let animes=match res.json::<AllAnimeSimple>().await {
                Ok(a) => a.anime,
                Err(e)=>{
                    dbg!(e);
                    vec![]
                }
               };
               anime.set(animes);
            }
     });
     ()
    });
    rsx!(
        div{
            id: "main_div_list_page",
            h2 { "{name}" },
            div { 
                id:"list_in_list_page",
                for entry in anime.read().clone(){
                    div { 
                        id:"Anime_card_list_page",
                        onclick: move |_| {
                            navigator.push(crate::frontend::router::routes::Details { id: entry.id.clone() });
                        },
                        img {
                            class: "dropdown_images_search",
                            loading: "eager",
                            src: entry.picture.clone().unwrap_or_else(|| "/assets/no_image.png".to_string()),
                            alt: "thumbanil"
                        },
                        span {
                            class: "span_items_search",
                            "{&entry.title}"
                        } 
                         
                    }       
                }
            }
        }
    )
}