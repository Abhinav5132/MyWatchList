use crate::frontend::manage_user_profile::choose_image;
pub use crate::frontend::*;

pub enum WatchListType {
    Public,
    Private,
    FriendsOnly,
}

impl WatchListType {
    pub fn string(&self) -> String {
        match self {
            WatchListType::Public => "Public".to_string(),
            WatchListType::Private => "Private".to_string(),
            WatchListType::FriendsOnly => "FriendsOnly".to_string(),
        }
    }
}

#[derive(Deserialize, Default)]
struct ACompleteList {
    name: String,
    image: String,
    is_ranked: i32,
    is_user_image: i32,
    privacy_type: String,
    description: String,
}

#[derive(Serialize, Debug)]
pub struct EditListPerUser {
    user_id: i64,
    list_id: i64,
    new_name: Option<String>,
    new_privacy_type: Option<String>,
    new_is_ranked: Option<i32>,
    new_image: Option<String>,
    is_user_image: Option<i32>,
    description: Option<String>,
}

// TODO change edit details to be the actual name of the list and clicking on it edits the name
#[component]
pub fn EditList(id: i64, on_close: EventHandler<()>) -> Element {
    let mut name = use_signal(|| "".to_string());
    let mut description = use_signal(|| "".to_string());
    let mut image = use_signal(|| "".to_string());
    let mut is_ranked = use_signal(|| -1);
    let mut is_user_image = use_signal(|| -1);
    let mut privacy_type = use_signal(|| "".to_string());

    let mut new_name = use_signal(|| None);
    let mut new_description = use_signal(|| None);
    let mut new_image = use_signal(|| None);
    let mut new_is_ranked = use_signal(|| None);
    let mut new_is_user_image = use_signal(|| None);
    let mut new_privacy_type = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            let client = Client::new();
            if let Ok(res) = client
                .get(format!(
                    "http://localhost:3000/get_list_details?list_id={}",
                    id
                ))
                .bearer_auth(TOKEN.read())
                .send()
                .await
                && let Ok(complete_list) = res.json::<ACompleteList>().await
            {
                name.set(complete_list.name);
                image.set(complete_list.image);
                is_ranked.set(complete_list.is_ranked);
                is_user_image.set(complete_list.is_user_image);
                privacy_type.set(complete_list.privacy_type);
                description.set(complete_list.description);
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
                                if let Some(link) = choose_image().await {
                                    image.set(link.clone());
                                    new_image.set(Some(link));
                                    new_is_user_image.set(Some(1));
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
                            new_name.set(Some(event.value()));

                        },
                        onkeydown: move |event| {
                            if event.code().to_string() == "ENTER" {
                                new_name.set(Some(name.read().to_string()));
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
                            new_description.set(Some(event.value()));
                        },
                        onkeydown: move |event| {
                            if event.code().to_string() == "ENTER" {
                                new_description.set(Some(description.read().to_string()));
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
                                    new_is_ranked.set(Some(1));  // only set if not already set
                                }
                            },
                            "Ranked"
                        }

                        button {
                            id:"Change_to_unranked",
                            onclick: move |_| {
                                if *is_ranked.read() == 1{
                                    is_ranked.set(0);
                                    new_is_ranked.set(Some(0));  // only set if true
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
                                new_privacy_type.set(Some(WatchListType::Public.string()));

                            },
                            "Public"
                        }

                        button {
                            id:"Change_to_private",
                            onclick: move |_| {
                                privacy_type.set(WatchListType::Private.string());
                                new_privacy_type.set(Some(WatchListType::Private.string()));

                            },
                            "Private"
                        }

                        button {
                            id:"Change_to_friendsonly",
                            onclick: move |_| {
                                privacy_type.set(WatchListType::FriendsOnly.string());
                                new_privacy_type.set(Some(WatchListType::FriendsOnly.string()));

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
                                let edit_list = EditListPerUser{
                                            new_image: new_image.read().clone(),
                                            user_id: *USERID.read(),
                                            list_id: id,
                                            new_name: new_name.read().clone(),
                                            new_is_ranked: *new_is_ranked.read(),
                                            new_privacy_type: new_privacy_type.read().clone(),
                                            is_user_image: *new_is_user_image.read(),
                                            description: new_description.read().clone()
                                        };
                                spawn(async move {
                                    if let Ok(res) = client.post("http://localhost:3000/edit-watch-list-from-user").json(
                                       &edit_list
                                    ).bearer_auth(TOKEN.read()).send().await {
                                        dbg!(edit_list);
                                        if res.status().is_server_error(){
                                            // use the status code do do error management later
                                        }
                                        on_close.call(());
                                    } else {
                                        dbg!("Failed to send edit list per user");
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
