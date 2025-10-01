use std::path::Path;
use crate::frontend::title_bar::UserDetails;
pub use crate::frontend::*;
use dioxus::desktop::tao::event;
use rfd::AsyncFileDialog;
pub use anyhow::Result;
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct ChangePfp{
    pfp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeUsername{
    user_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePassword{
    pwd: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeEmail{
    email: String,
}

#[component]
pub fn ManageAccount() -> Element{
    let mut username = use_signal(|| "".to_string());
    let mut user_email = use_signal(|| "".to_string());
    let mut user_pfp = use_signal(|| "".to_string());
    let mut refetch_title = use_context::<Signal<bool>>();

    let mut edit_username_trigger = use_signal(|| true);
    let mut edit_email_trigger = use_signal(|| true);
    let mut edit_pwd_trigger = use_signal(|| true);

    use_effect(move || {
        //let mut _a = refetch_signal.read();
        let _ = refetch_title.read();
        let user_name = USERNAME.cloned();
        let user_id = USERID.cloned();
        if user_id != -1 && user_name != "".to_string(){ 
            spawn (async move{
                let client = Client::new();
                if let Ok(res) = client.post("http://localhost:3000/get_user_details").json(&json!({ // can posibly turned into a macro
                    "user_id": user_id,
                })).send().await {
                    if let Ok(usr_dets) = res.json::<UserDetails>().await{
                        username.set(usr_dets.username);
                        user_email.set(usr_dets.user_email);
                        user_pfp.set(usr_dets.user_pfp);
                    }
                }
            });
        }}
    );

    let mut submit_username = {
        let username = username.clone();
        let mut refetch_title = refetch_title.clone();

        move || {
            edit_username_trigger.set(true); // lock field again

            spawn(async move {
                let client = Client::new();
                if let Ok(_res) = client
                    .post("http://localhost:3000/change_username")
                    .json(&ChangeUsername {
                        user_name: username.read().to_string(),
                    })
                    .bearer_auth(TOKEN.read())
                    .send()
                    .await
                {
                    dbg!("Username changed (via submit)");
                    let current = !*refetch_title.read();
                    refetch_title.set(current);
                }
            });
        }
    };

    let mut submit_email = {
        let email = user_email.clone();
        let mut refetch_title = refetch_title.clone();

        move || {
            edit_email_trigger.set(true); // lock field again

            spawn(async move {
                let client = Client::new();
                if let Ok(_res) = client
                    .post("http://localhost:3000/change_email")
                    .json(&ChangeEmail {
                        email: email.read().to_string(),
                    })
                    .bearer_auth(TOKEN.read())
                    .send()
                    .await
                {
                    dbg!("email changed (via submit)");
                    let current = !*refetch_title.read();
                    refetch_title.set(current);
                }
            });
        }
    };


    rsx!(

        div { 
            id: "Profile_picture_div",
            onclick: move |_| {
                spawn(async move{
                    if let Some(blob) = choose_image().await {
                        let client = Client::new();
                        if let Ok(res) = client.post("http://localhost:3000/change_pfp").json(
                            &ChangePfp{
                                pfp: base64::engine::general_purpose::STANDARD.encode(blob)
                            }
                        )
                        .bearer_auth(TOKEN.read()).send().await {
                            //let current_refresh = *refetch_signal.read();
                            let current = !*refetch_title.read();
                            refetch_title.set(current);
                        }
                    }
                });
            },
            img { 
                id:"Profile_picture",
                src: user_pfp,
                alt: "user_image"
            },
        }

        div { 
            id:"User_details_div",
            h2 { "Account Details:" }

            div { 
                id: "Username_div",
                label { "Username:" }
                input { 
                    class:"change_details_div",
                    r#type: "text",
                    value: "{ username }",
                    disabled: edit_username_trigger, // change so only disable till the user clcicks the button
                    oninput: move |evt| {
                        username.set(evt.value());
                    },
                    onkeydown: move |evt| {
                        if evt.code().to_string() == "Enter".to_string() {
                            submit_username();
                        } 
                    }
                }
                
                if *edit_username_trigger.read(){
                    button { 
                        class: "confirm_change_buttons",
                        onclick: move |_| {
                            edit_username_trigger.set(false);
                        },
                        "Edit"
                    }
                } else {
                    button { 
                        class: "confirm_change_buttons",
                        onclick: move |_| submit_username(),
                        "Submit"
                    }
                }
            }

            div { 
                id: "email_div",
                label { "Email:" }
                input { 
                    class:"change_details_div",
                    r#type: "text",
                    value: "{ user_email }",
                    disabled: edit_email_trigger, // change so only disable till the user clcicks the button
                    oninput: move |evt| {
                        user_email.set(evt.value());
                    },
                    onkeydown: move |evt| {
                        if evt.code().to_string() == "Enter".to_string() {
                            submit_email();
                        } 
                    }
                }
                
                if *edit_username_trigger.read(){
                    button { 
                        class: "confirm_change_buttons",
                        onclick: move |_| {
                            edit_email_trigger.set(false);
                        },
                        "Edit"
                    }
                } else {
                    button { 
                        class: "confirm_change_buttons",
                        onclick: move |_| submit_email(),
                        "Submit"
                    }
                }
            }
         }
    )
}

pub async fn choose_image() -> Option<Vec<u8>> {
    let new_image = AsyncFileDialog::new().add_filter("pictures", &["png", "jpg", "jpeg"]).
    pick_file().await;

    if let Some(image) = new_image{
        let path = image.path();
        let blob = match file_to_blob_with_path(path)
        {
            Ok(blob) => Some(blob),
            Err(e) => {
                dbg!(e);
                None
            }
        };
        return blob;
    }
    None
}

pub fn file_to_blob_with_path(path: &Path) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    Ok(bytes)
}
