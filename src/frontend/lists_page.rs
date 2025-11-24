use reqwest::{Client, StatusCode};
use crate::frontend::popup_edit_list::EditList;
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
                                    navigator.push(crate::frontend::router::routes::ListPgFn { list_name: li.name.clone(), user_id: user_id });
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