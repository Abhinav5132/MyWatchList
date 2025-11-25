pub use crate::frontend::*;
use crate::frontend::{
    lists_page::{AList, AllListSimple, FetchLists},
    popup_add_anime::{PopupAddAnime, PopupError},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct FullAnimeResult {
    title_romanji: String,
    format: String,
    description: String,
    episodes: i32,
    status:String,
    anime_season: String,
    anime_year: i32,
    largeImage: String,
    duration: i32,
    score: f32,
    studio: Option<Vec<String>>,
    synonyms: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    recommendations: Vec<ReccomendResult>,
    related_anime: Vec<RelatedAnime>
}

#[derive(Serialize, Default, Deserialize, PartialEq, Clone, Debug)]
pub struct RelatedAnime{
    title_romanji: String,
    id: i64,
    picture: String,
    relationType: String,
}

#[derive(Serialize, Default, Deserialize, PartialEq, Clone, Debug)]
pub struct ReccomendResult{
    id: i32,
    title: String,
    picture: String,
    score: f32,
}

#[derive(Serialize)]
struct AddToList {
    anime_id: i64,
    list_id: i64,
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
    list_id: i64,
    user_id: i64,
}

#[derive(Deserialize)]
pub struct IsRanked {
    is_ranked: i32,
    last_rank: i32,
}

pub async fn check_if_in_list(id: i64, list_name: String, list_id: i64) -> bool {
    let client = Client::new();
    if let Ok(resp) = client
        .get("http://localhost:3000/check_if_already_in_list")
        .json(&AddToList {
            list_id:list_id,
            anime_id: id.clone(),
            list_name: list_name,
            user_id: *USERID.read(),
            rank: None,
        })
        .send()
        .await
    {
        if let Ok(count) = resp.json::<ExistsInList>().await {
            if count.exists { true } else { false }
        } else {
            false
        }
    } else {
        false
    }
}

pub async fn check_if_list_is_ranked(list_id: i64, user_id: i64) -> IsRanked {
    let client = Client::new();

    if let Ok(resp) = client
        .get("http://localhost:3000/get-if-ranked")
        .json(&IfRanked {
            list_id: list_id,
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

pub async fn add_anime_to_list(id: i64, list_name: String, rank: Option<i32>, list_id: i64) -> bool {
    if !check_if_in_list(id.clone(), list_name.clone(), list_id.clone()).await {
        let client = Client::new();
        let userid = *USERID.read();
        if let Ok(resp) = client
            .post("http://localhost:3000/add-anime-to-list")
            .json(&AddToList {
                anime_id: id,
                user_id: userid,
                list_id: list_id,
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

pub async fn get_all_lists() -> Option<(AllListSimple, i64, i64)> {
    let client = Client::new();

    if let Ok(res) = client
        .get("http://localhost:3000/fetch-all-lists")
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
                let recommend_id = lists.list.iter().find(|l| l.name == "Recommended").unwrap_or(&AList::default()).id;
                let watch_id = lists.list.iter().find(|l| l.name == "Watch_List").unwrap_or(&AList::default()).id;
                lists.list = lists
                    .list
                    .into_iter()
                    .filter(|l| l.name != "Recommended" && l.name != "Watch_List")
                    .collect();

                Some((lists, recommend_id, watch_id))
            }
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
    let mut all_lists: Signal<AllListSimple> = use_signal(|| AllListSimple::default());
    let mut show_list = use_signal(|| false);
    let mut list_id = use_signal(|| 0i64);
    let navigator = use_navigator();
    let mut recommend_id = use_signal(|| 0i64);
    let mut watch_id = use_signal(|| 0i64);

    use_effect(move || {
        let mut details = anime_details.clone();
        spawn(async move {
            let client = Client::new();
            if let Ok(res) = client
                .get(format!(
                    "http://localhost:3000/details?query={}",
                    id.clone()
                ))
                .send()
                .await
            {
                /*if let Ok(detail) = res.json::<FullAnimeResult>().await {
                    details.set(Some(detail));
                } else {
                    dbg!("Failed to decode into json");
                }*/

                match res.json::<FullAnimeResult>().await{
                    Ok(detail) => {
                        details.set(Some(detail));
                    }
                    Err(e) => {
                        dbg!(e);
                    }
                }
            }
            if let Some(lists) = get_all_lists().await{
                all_lists.set(lists.0);
                recommend_id.set(lists.1);
                watch_id.set(lists.2);
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
                            src: "{ details.largeImage }",
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
                                                let status = add_anime_to_list(id.clone(), list_name.read().clone(), Some(rank), *list_id.read()).await;
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
                                    list_id.set(*recommend_id.read());

                                    spawn(async move {
                                        dbg!(list_id);
                                        let rank_status = check_if_list_is_ranked(*list_id.read(), *USERID.read()).await;
                                        if rank_status.is_ranked == 0 {
                                            // not ranked
                                            is_ranked.set(false);
                                            last_rank.set(0); // this should be null
                                        }
                                        else if rank_status.is_ranked == 1 {
                                            is_ranked.set(true);
                                            last_rank.set(rank_status.last_rank);  
                                        }
                                        show_popup.set(true);
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
                                    
                                    for alist in all_lists.read().list.clone() { // only prints out watch_list for some reaon
                                        div {
                                            class: "a_list_div",
                                            onclick: move |_| {
                                                list_name.set(alist.name.clone());
                                                list_id.set(alist.id);
                                                spawn(async move {
                                                    dbg!("this code ran");
                                                    let is_rank = check_if_list_is_ranked(*list_id.read(), *USERID.read()).await;
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
                                            "{related.title_romanji}",
                                            "{related.relationType}"
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
                navigator.push(crate::frontend::router::routes::HomePage {  });
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
