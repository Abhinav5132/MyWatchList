use reqwest::{Client, StatusCode};
use crate::frontend::{manage_user_profile::choose_image, popup_edit_list::{EditList, WatchListType}};
pub use crate::frontend::*;

#[derive(Serialize)]
pub struct FetchLists{
    pub user_id: i64,
    pub page_no: i32,
    pub per_page: i32
}
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct AList{
    pub name: String,
    pub id: i64,
    pub image: String,
    pub description: String,
}

#[derive(Deserialize, Clone, Default)]
pub struct AllListSimple{
    pub list: Vec<AList>

}

#[derive(Serialize)]
pub struct AddListToUser{

    user_id: i64,
    name: String,
    privacy_type: String,
    is_ranked: i32,
    image: String,
    is_user_image: i32,
    description: String,
}

#[derive(Serialize, Debug)]
pub struct EditListPerUser{
    user_id: i64,
    list_id: i64,
    new_name: Option<String>,
    new_privacy_type: Option<String>,
    new_is_ranked: Option<i32>,
    new_image: Option<String>,
    is_user_image: Option<i32>,
    description: Option<String>
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
    let mut show_delete_popup: Signal<(bool, i64)> = use_signal(|| (false, -1));
    let mut show_add_new_list_popup: Signal<bool> = use_signal(|| false);
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
                        div {
                            span {
                                class: "span_items_search",
                                "{li.name}"
                            } 
                            img {
                                class: "dropdown_images_search",
                                loading: "eager",
                                src: li.image,
                                alt: "thumbanil",
                                onclick: move |_| {
                                    navigator.push(crate::frontend::router::routes::ListPgFn { list_id: li.id, user_id: user_id });
                                },
                            }, 
                        }
                        
                        div {  
                            p { 
                                "{li.description}"
                            }
                            div {
                                class: "button_row",
                                button { 
                                        class:"edit_buttons",
                                    onclick: move |_| {
                                        // add a popup allowing them to change the details of the list, add custom images ect
                                        show_edit_list.set(true);
                                        edit_which_list.set(li.id);
                                    },
                                    "Edit list"
                                }

                                button { 
                                    class:"delete_list_button",
                                    onclick: move |_| {
                                        show_delete_popup.set((true, li.id.clone()));
                                    },
                                    img {
                                        class:"Feeling_icon",
                                        src: TRAHSH
                                    }
                                }
                            }
                            
                        }
                    }
                }
            }

            button { 
                id:"add_new_list_button",
                onclick: move |_| {
                    show_add_new_list_popup.set(true);
                },
                "Add new list"
            }
        }
        if *show_add_new_list_popup.read() {
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

                    show_add_new_list { 
                        onClose: move |_| {
                        show_add_new_list_popup.set(false);
                        }
                    }
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

        if show_delete_popup.read().0 && show_delete_popup.read().1 != -1 {

            list_delete_pop_up { 
                list_id: show_delete_popup.read().1,
                onClose: move |_| {
                    show_delete_popup.set((false, -1));
                }
            }
        }
    )
}
// TODO change edit details to be the actual name of the list and clicking on it edits the name

#[component]
pub fn list_delete_pop_up(onClose: EventHandler, list_id: i64)-> Element {
    let a_string = format!("Are you sure you want to delete {}", "idk" );
    rsx!(
        div { 
            id:"really_delete_button",
            h3 { "{a_string}" },
            button { 
                class: "delete_list_button",
                onclick: move |_| {
                    let client = Client::new();

                    spawn(async move {
                        if let Ok(req) = client.post(format!("http://localhost:3000/delete_list?list_id={}", list_id))
                        .bearer_auth(TOKEN.read()).send().await{
                            if req.status() != StatusCode::OK {
                                // something wrong happened here.
                                dbg!(req.status());
                            }
                            onClose.call(())
                        }
                        else{
                            // failed to send the request 
                            onClose.call(())
                        }
                    });
                },

                "DELETE"
            }

            button { 
                class: "cancel_list_button",
                onclick: move |_| {
                    onClose.call(())
                },
                "Cancel"
            }
        }
    )
}

#[component]
pub fn show_add_new_list(onClose: EventHandler) -> Element {
    let mut name = use_signal(|| "".to_string());
    let mut description = use_signal(|| "".to_string());
    let mut image = use_signal(|| "".to_string());
    let mut is_ranked = use_signal(|| 0);
    let mut privacy = use_signal(|| WatchListType::Public);
    let mut is_user_image = use_signal(|| 0);
    rsx!(
        div {
            id: "edit_list_details",
            h2 { "Create new list" }
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
                                if let Some(link) = choose_image().await {
                                    image.set(link.clone());
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
                                    is_ranked.set(1);
                                }
                            },
                            "Ranked"
                        }

                        button { 
                            id:"Change_to_unranked",
                            onclick: move |_| {
                                if *is_ranked.read() == 1{
                                    is_ranked.set(0);
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
                                privacy.set(WatchListType::Public);  
                                
                            },
                            "Public"
                        }

                        button { 
                            id:"Change_to_private",
                            onclick: move |_| {
                                privacy.set(WatchListType::Private); 

                            },
                            "Private"
                        }

                        button { 
                            id:"Change_to_friendsonly",
                            onclick: move |_| {
                                privacy.set(WatchListType::FriendsOnly); 

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
                                onClose.call(());
                            },
                            "Cancel"
                        }

                        button { 
                            id:"Submit_button_edit_list",
                            onclick: move |_| {
                                let client = Client::new();
                                spawn(async move {
                                    if let Ok(req) = client.post("http://localhost:3000/add-list-to-user")
                                    .bearer_auth(TOKEN.read()).json(&AddListToUser{
                                        user_id: *USERID.read(),
                                        is_user_image: *is_user_image.read(),
                                        name: name.read().to_string(),
                                        description: description.read().to_string(),
                                        privacy_type: privacy.read().string(),
                                        image: image.read().to_string(),
                                        is_ranked: *is_ranked.read()
                                    }).send().await {
                                        // do some actual error handeling here 
                                        onClose.call(());
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