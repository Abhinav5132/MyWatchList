use reqwest::Client;
use crate::frontend::manage_user_profile::choose_image;
pub use crate::frontend::*;

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
    pub image: String,
}

#[derive(Deserialize, Clone)]
pub struct AllListSimple{
    pub list: Vec<AList>

}

#[derive(Deserialize, Default)]
struct ACompleteList{
    name: String,
    image: String,
    is_ranked: bool,
    is_user_image: bool,
    privacy_type: String
}

// TODO add actual error checking here, if user id is -1 unauthorized
#[component]
pub fn ListsPgFn(user_id: i64) -> Element{
    let page = use_signal(|| 1);
    let mut all_list = use_signal(|| AllListSimple{
        list: vec![]
    });
    let navigator = use_navigator();
    let mut show_edit_list = use_signal(|| false);
    use_future(move || async move {
        let client = match Client::builder().danger_accept_invalid_certs(true).build() {
            Ok(c) => c,
            Err(e) => {
                dbg!(e);
                panic!("Failed to build a client") //.expect
            }
        };
        if let Ok(res) = client.get("http://localhost:3000/fetch-all-lists")
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
           all_list.set(lists);

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
                            navigator.push(crate::frontend::router::routes::ListPgFn { list_name: li.name.clone(), user_id: user_id });
                        },
                        img {
                            class: "dropdown_images_search",
                            loading: "eager",
                            src: li.image,
                            alt: "thumbanil"
                        },
                        span {
                            class: "span_items_search",
                            "{li.name}"
                        } 

                        button { 
                            class:"edit_buttons",
                            onclick: move |_| {
                                // add a popup allowing them to change the details of the list, add custom images ect
                                show_edit_list.set(true);
                            },
                            "Edit list"
                         }
                    }
                }
            }

            button { 
                id:"add_new_list_button",
                onclick: move |_| {

                }
             }
        }

        if *show_edit_list.read() {
            EditList {  }
        }
    )
}

#[component]
pub fn EditList( ) -> Element {

    let mut name = use_signal(|| "".to_string());
    let mut image = use_signal(|| "".to_string());
    let mut is_ranked= use_signal(|| false);
    let mut is_user_image = use_signal(|| false);
    let mut privacy_type= use_signal(|| "".to_string());

    use_effect(move || {
        spawn(async move{
            let client = Client::new();
            if let Ok(res) = client.get("http://localhost:3000/get_list_details")
            .bearer_auth(TOKEN.read()).send().await{
                if let Ok(complete_list) = res.json::<ACompleteList>().await{
                   name.set(complete_list.name);
                   image.set(complete_list.image);
                   is_ranked.set(complete_list.is_ranked);
                   is_user_image.set(complete_list.is_user_image);
                   privacy_type.set(complete_list.privacy_type);
                   
                } 
            }
        });
    });

    rsx!(

        div { 
            id: "edit_list_details",
            h2 { "Edit Details" }
            div {
                id:"edit_details_div",  
                div { 
                    id: "edit_list_image", 
                    img { 
                        id:"list_image_edit",
                        src: image.read().to_string(),
                        alt:"list image",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(blob) = choose_image().await {
                                    let base_64_img = base64::engine::general_purpose::STANDARD.encode(blob);
                                    let data_url = format!("data:image/png;base64,{}", base_64_img);
                                    image.set(data_url);
                                    is_user_image.set(true);
                                }
                            });
                        }
                    },
                    
                }

                div { 
                    id:"edit_other_details",
                    input { 
                        id: "edit_list_name",

                     }
                }
            }
         }

    )
}