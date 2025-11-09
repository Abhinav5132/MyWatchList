use dioxus::html::script::r#async;
use reqwest::Client;
use crate::{backend::add_to_list::WatchListType, frontend::manage_user_profile::choose_image};
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
    pub description: String,
}

#[derive(Deserialize, Clone)]
pub struct AllListSimple{
    pub list: Vec<AList>

}

#[derive(Deserialize, Default)]
struct ACompleteList{
    name: String,
    image: String,
    is_ranked: i32,
    is_user_image: i32,
    privacy_type: String,
    description: String
}

#[derive(Serialize)]
pub struct EditListPerUser{
    user_id: i64,
    list_id: i64,
    new_name: String,
    new_privacy_type: String,
    new_is_ranked: i32,
    new_image: String,
    is_user_image: i32,
    description: String
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
    let mut edit_which_list:Signal<i64> = use_signal(|| -1);
    use_future(move || async move {
        let client = Client::new();
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
                       
                        img {
                            class: "dropdown_images_search",
                            loading: "eager",
                            src: li.image,
                            alt: "thumbanil",
                            onclick: move |_| {
                                navigator.push(crate::frontend::router::routes::ListPgFn { list_name: li.name.clone(), user_id: user_id });
                            },
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
                                edit_which_list.set(li.id);
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

        if *show_edit_list.read() && *edit_which_list.read() != -1 {
            div {
                id: "edit_list_modal_overlay",
                onclick: move |_| {
                    show_edit_list.set(false);
                },

                // Stop clicks inside the popup from closing it
                div {
                    id: "edit_list_modal_content",
                    onclick: move |event| {
                        event.stop_propagation();
                    },

                    EditList {
                        id: *edit_which_list.read(),
                        on_close: move |_| {
                            show_edit_list.set(false);
                            edit_which_list.set(-1);
                        }
                    }
                }
            }
        }
    )
}
// TODO change edit details to be the actual name of the list and clicking on it edits the name
#[component]
pub fn EditList(id: i64, on_close: EventHandler<()>) -> Element {

    let mut name = use_signal(|| "".to_string());
    let mut description = use_signal(|| "".to_string());
    let mut image = use_signal(|| "".to_string());
    let mut is_ranked= use_signal(|| -1);
    let mut is_user_image = use_signal(|| -1);
    let mut privacy_type= use_signal(|| "".to_string());

    use_effect(move || {
        spawn(async move{
            let client = Client::new();
            if let Ok(res) = client.get(format!("http://localhost:3000/get_list_details?list_id={}", id))
            .bearer_auth(TOKEN.read()).send().await{
                if let Ok(complete_list) = res.json::<ACompleteList>().await{
                   name.set(complete_list.name);
                   image.set(complete_list.image);
                   is_ranked.set(complete_list.is_ranked);
                   is_user_image.set(complete_list.is_user_image);
                   privacy_type.set(complete_list.privacy_type);
                   description.set(complete_list.description);
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
                                    is_user_image.set(1);
                                }
                            });
                        }
                    },
                    
                }

                div { 
                    id:"edit_other_details",
                    input { 
                        id: "edit_list_name",
                        r#type: "text",
                        value: name,
                        placeholder: "Enter name here:",
                        oninput: move |event| {
                            name.set(event.value());
                        },
                        onkeydown: move |event| {
                            if event.code().to_string() == "ENTER".to_string() {
                                // name is set move focus to the next field
                            }
                        }
                    }

                    input { 
                        id: "edit_list_description",
                        r#type: "text",
                        placeholder:"Enter description here",
                        value: description,
                        oninput: move |event| {
                            description.set(event.value());
                        },
                        onkeydown: move |event| {
                            if event.code().to_string() == "ENTER".to_string() {
                                    // name is set move focus to the next field
                            }
                        } 
                    }
                    // TODO: needs to do a readjusting of the watch list, if ranked it should rank based on date added, else the ranks should be set to null and sorted by date
                    label{"Sorting:"}
                    div { 
                        id:"Ranked_choices_button",
                        button { 
                            id:"Change_to_ranked",
                            onclick: move |_| {
                                if *is_ranked.read() != 1 {
                                    is_ranked.set(1);  // only set if not already set 
                                }
                            },
                            "Ranked"
                        }

                        button { 
                            id:"Change_to_unranked",
                            onclick: move |_| {
                                if *is_ranked.read() == 1{
                                    is_ranked.set(0);  // only set if true
                                }

                            },
                            "Unranked"
                        }
                    }

                    label{"Privacy:"}
                    div { 
                        id:"privacy_choices",
                        button { 
                            id:"set_to_public",
                            onclick: move |_| {
                                privacy_type.set(WatchListType::Public.string());  
                                
                            },
                            "Public"
                        }

                        button { 
                            id:"Change_to_private",
                            onclick: move |_| {
                                privacy_type.set(WatchListType::Private.string()); 
                            },
                            "Private"
                        }

                        button { 
                            id:"Change_to_friendsonly",
                            onclick: move |_| {
                                privacy_type.set(WatchListType::FriendsOnly.string());  
                            },
                            "Friends Only"
                        }
                    }
                    // TODO submit button and integration with the back end

                    div { 
                        id:"Submit_close_buttons",
                        button { 
                            id: "close_button_edit_list",
                            onclick: move |_| {
                                on_close.call(());
                            },
                            "Cancel"
                        }

                        button { 
                            id:"Submit_button_edit_list",
                            onclick: move |_| {
                                let client = Client::new();
                                spawn(async move {
                                    if let Ok(res) = client.post("http://localhost:3000/edit-watch-list-from-user").json(
                                        &EditListPerUser{
                                            new_image: image.read().to_string(),
                                            user_id: *USERID.read(),
                                            list_id: id.clone(),
                                            new_name: name.read().to_string(),
                                            new_is_ranked: *is_ranked.read(),
                                            new_privacy_type: privacy_type.read().to_string(),
                                            is_user_image: *is_user_image.read(),
                                            description: description.read().to_string()
                                        }
                                    ).bearer_auth(TOKEN.read()).send().await {
                                        if res.status().is_server_error(){
                                            // use the status code do do error management later 
                                        }
                                        on_close.call(());
                                    }
                                });
                            },
                            "Submit"
                        }
                    }
                }
            }
        }

    )
}