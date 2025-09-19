

pub use crate::*;
use crate::{
    lists_page::{AllListSimple, FetchLists}, popup_add_anime::{PopupAddAnime, PopupError}, *
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
pub use tracing;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct FullAnimeResult {
    title_romanji: String,
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
    recommendations: Vec<ReccomendResult>,
    related_anime: Vec<RelatedAnime>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ReccomendResult {
    id: i32,
    title: String,
    picture: String,
    score: f32,
}
#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
struct RelatedAnime {
    id: i32,
    title: String,
    picture: String,
    RelationType: String,
}

#[derive(Serialize)]
struct AddToList {
    anime_id: i64,
    list_name: String,
    user_id: i64,
    rank: Option<i32>,
}

#[derive(Deserialize, Serialize)]
pub struct ExistsInList {
    exists: bool,
}

#[derive(Serialize)]
pub struct IfRanked {
    list_name: String,
    user_id: i64,
}

#[derive(Deserialize)]
pub struct IsRanked {
    is_ranked: i32,
    last_rank: i32,
}

pub async fn check_if_in_list(id: i64, list_name: String) -> bool {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build client");
    if let Ok(resp) = client
        .get("/check_if_already_in_list")
        .json(&AddToList {
            anime_id: id.clone(),
            list_name: list_name,
            user_id: *USERID.read(),
            rank: None,
        })
        .send()
        .await
    {
        if let Ok(count) = resp.json::<ExistsInList>().await {
            if count.exists {
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

pub async fn check_if_list_is_ranked(list_name: String, user_id: i64) -> IsRanked {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build client");

    if let Ok(resp) = client
        .get("https://localhost:3000/get-if-ranked")
        .json(&IfRanked {
            list_name: list_name,
            user_id: user_id,
        })
        .bearer_auth(TOKEN.read())
        .send()
        .await
    {
        if let Ok(is_ranked) = resp.json::<IsRanked>().await {
            is_ranked
        } else {
            IsRanked {
                is_ranked: -1,
                last_rank: 1,
            } // for now till i figure out how result types work
        }
    } else {
        IsRanked {
            is_ranked: -1,
            last_rank: 1,
        } // again for now
    }
}

pub async fn add_anime_to_list(id: i64, list_name: String, rank: Option<i32>) -> bool {
    if !check_if_in_list(id.clone(), list_name.clone()).await {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to build client");
        let userid = *USERID.read();
        if let Ok(resp) = client
            .post("https://localhost:3000/add-anime-to-list")
            .json(&AddToList {
                anime_id: id,
                user_id: userid,
                list_name: list_name,
                rank: rank,
            })
            .bearer_auth(TOKEN.read())
            .send()
            .await
        {
            let status = resp.status();
            if status.is_server_error() {
                false
            } else {
                true
            }
        } else {
            false
        }
    } else {
        true // anime already in list so nothing needs to be changed(later change so if already in the list the user cannot send that request)
    }
}

pub async fn get_all_lists() -> Option<AllListSimple> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build client.");

    if let Ok(res) = client
        .get("https://localhost:3000/fetch-all-lists")
        .bearer_auth(TOKEN.read())
        .json(&FetchLists {
            user_id: *USERID.read(),
            page_no: 1,
            per_page: 5,
        })
        .send()
        .await
    {
        let alllist = res.json::<AllListSimple>().await;
        match alllist {
            Ok(mut lists) => {
               lists.list = lists
                .list
                .into_iter()
                .filter(|l| l.name != "Recommended" && l.name != "Watch_List")
                .collect();

                Some(lists)
            },
            Err(e) => {
                dbg!(e);
                None
            }
        }
    } else {
        dbg!("didnt gget a response from backend");
        None
    }
}

#[component]
pub fn Details(id: i64) -> Element {
    let mut show_popup: Signal<bool> = use_signal(|| false);
    let mut pop_error: Signal<bool> = use_signal(|| false);
    let anime_details: Signal<Option<FullAnimeResult>> = use_signal(|| None);
    let mut is_ranked = use_signal(|| false);
    let mut last_rank = use_signal(|| 0);
    let mut list_name = use_signal(|| "".to_string());
    let mut all_lists: Signal<Option<AllListSimple>> = use_signal(|| None);
    let mut show_list = use_signal(|| false);
    let navigator = use_navigator();
    use_effect(move || {
        let mut details = anime_details.clone();
        spawn(async move {
            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .expect("Failed to build client");
            if let Ok(res) = client
                .get(format!(
                    "https://localhost:3000/details?query={}",
                    id.clone()
                ))
                .send()
                .await
            {
                if let Ok(detail) = res.json::<FullAnimeResult>().await {
                    details.set(Some(detail));
                }
            }
            let option_lists = get_all_lists().await;
            all_lists.set(option_lists);
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

                div{
                    id:"Title_div",
                h3 { id: "Title",
                    "{ details.title_romanji }" },
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

                            if *show_popup.read(){
                                div {
                                    id: "popup_anime_overlay",
                                    PopupAddAnime {
                                        anime_name: &details.title_romanji,
                                        list_name:list_name.read(), // change to a signal
                                        last_rank:*last_rank.read(),
                                        is_rank: *is_ranked.read(),
                                        on_close: move  || {
                                            show_popup.set(false);
                                        },
                                        on_submit: move |rank:i32| {
                                            spawn({async move {
                                                let status = add_anime_to_list(id.clone(), list_name.read().clone(), Some(rank)).await;
                                                pop_error.set(!status);
                                            }
                                        });

                                        }
                                    }
                                }
                            }

                            if *pop_error.read(){
                                div {
                                    id: "popup_error",
                                    PopupError {
                                        anime_name: &details.title_romanji,
                                        list_name:list_name.read(), // change to a signal
                                        on_close: move || {
                                            pop_error.set(false);
                                        }
                                     }
                                 }
                            }


                        div {
                            id: "Like_button_div",
                            button {
                                id:"Recommend_button",
                                onclick: move |_| {
                                    list_name.set("Recommended".to_string());

                                    spawn(async move {

                                        let rank_status = check_if_list_is_ranked("Recommended".to_string(), *USERID.read()).await;
                                        if rank_status.is_ranked == 0 {
                                            // not ranked
                                            is_ranked.set(false);
                                            last_rank.set(0); // this should be null
                                            show_popup.set(true);
                                        }
                                        else if rank_status.is_ranked == 1 {
                                            is_ranked.set(true);
                                            last_rank.set(rank_status.last_rank);
                                            show_popup.set(true);


                                        }
                                        else{
                                            // this is an error change things here.
                                        }

                                    });

                                },
                                img {
                                    class:"Feeling_icon",
                                    src:HEART,
                                }
                                "Recommend"
                            }
                            button {
                                id:"Watch_list_button",
                                img {
                                    class:"Feeling_icon",
                                    id:"Add",
                                    src:ADD
                                }
                                "Add to list"
                                }

                            div {
                                id:"split_list_button",
                                button {
                                        id:"Multiple_list_button",
                                        onclick: move |_| {
                                            let current  = *show_list.read();
                                            show_list.set(!current);
                                        },
                                        img {
                                            class:"Feeling_icon",
                                            id:"Add",
                                            src:ADD
                                        }
                                    }
                                    if *show_list.read() {
                                    div {
                                    id:"dropdown_of_lists",
                                    if let Some(all_list) = all_lists.read().as_ref(){
                                        
                                        for alist in all_list.list.clone() { // on)ly prints out watch_list for some reaon

                                            div {
                                                class: "a_list_div",
                                                onclick: move |_| {
                                                    let current_list_name = alist.name.clone();
                                                    list_name.set(alist.name.clone());
                                                    spawn(async move {
                                                        dbg!("this code ran");
                                                        let is_rank = check_if_list_is_ranked(current_list_name, *USERID.read()).await;
                                                        if is_rank.is_ranked == 0{
                                                            is_ranked.set(false);
                                                            last_rank.set(0);
                                                            show_popup.set(true);
                                                        }
                                                        else if is_rank.is_ranked == 1 {
                                                            is_ranked.set(false);
                                                            last_rank.set(is_rank.last_rank);
                                                            show_popup.set(true);
                                                        } else {
                                                            dbg!("Unexpected is_ranked value: {}", is_rank.is_ranked);
                                                        }
                                                    });
                                                },
                                                p { { alist.name.clone()} }
                                            }
                                        }
                                    }

                                }
                                }
                                }


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
                        div{
                            id:"other_anime_div",
                            div {
                                id: "Recommendations_div",
                                for reccommend in details.recommendations.clone(){
                                    div {
                                        class: "Recommend_item_div",
                                        img {
                                            id: "Reccomend_pic",
                                            src:reccommend.picture
                                        }
                                        div {
                                            id: "Recc_title_div",
                                            "{reccommend.title}",
                                            "{reccommend.score / 10.0}"
                                        }
                                    }
                                }
                            }

                            div {
                                id:"Related_div",
                                for related in details.related_anime.clone(){
                                    div {
                                        class:"Related_item_div",
                                        img{
                                            id: "related_pic",
                                            src: related.picture
                                        }
                                        div {
                                            id:"rel_title_div",
                                            "{related.title}",
                                            "{related.RelationType}"
                                        }
                                    }
                                }
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
            "Back"
        }},


            None => rsx!{
                h1 { "Loading" }
            }
        }
    }
}
