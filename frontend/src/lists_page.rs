
use reqwest::Client;
pub use crate::*;

#[derive(Serialize)]
pub struct FetchLists{
    pub user_id: i64,
    pub page_no: i32,
    pub per_page: i32
}
#[derive(Deserialize, Clone, Debug)]
pub struct AList{
    pub name: String,
    pub id: i64,
    pub picture: Vec<u8>,
}

#[derive(Deserialize, Clone)]
pub struct AllListSimple{
    pub list: Vec<AList>
}

// TODO add actual error checking here, if user id is -1 unauthorized
#[component]
pub fn ListsPgFn(user_id: i64) -> Element{
    let page = use_signal(|| 1);
    let mut all_list = use_signal(|| AllListSimple{
        list: vec![]
    });
    let navigator = use_navigator();
    use_future(move || async move {
        let client = match Client::builder().danger_accept_invalid_certs(true).build() {
            Ok(c) => c,
            Err(e) => {
                dbg!(e);
                panic!("Failed to build a client") //.expect
            }
        };
        if let Ok(res) = client.get("https://localhost:3000/fetch-all-lists")
        .bearer_auth(TOKEN.read())
        .json(
            &FetchLists{
                user_id: user_id,
                page_no: *page.read(),
                per_page: 10 // change this value later
            }
        )
        .send()
        .await
        {
           let lists = match res.json::<AllListSimple>().await{
            Ok(list) => list,
            Err(e) => {
                dbg!(e);
                AllListSimple{
                    list: vec![]
                }
            }
           };

           all_list.set(lists)
        }

    });
    rsx!(
         div{
            id: "main_div_list_page",
            h2 { "Lists" }, 
            div {  
                for li in all_list.read().list.clone() {
                    div{
                        id:"Anime_card_list_page",
                        onclick: move |_| {
                            navigator.push(crate::router::routes::ListPgFn { list_name: li.name.clone(), user_id: user_id });
                        },
                        img {
                            class: "dropdown_images_search",
                            loading: "eager",
                            src: entry.picture.clone().unwrap_or_else(|| "/assets/no_image.png".to_string()),
                            alt: "thumbanil"
                        },
                        span {
                            class: "span_items_search",
                            "{li.name}"
                        } 
                    }
                }
            }
        }
    )
}