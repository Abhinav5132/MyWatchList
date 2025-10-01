use std::path::Path;
use crate::frontend::title_bar::UserDetails;
pub use crate::frontend::*;
use rfd::AsyncFileDialog;
pub use anyhow::Result;
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct ChangePfp{
    pfp: String,
}

#[component]
pub fn ManageAccount() -> Element{
    let mut username = use_signal(|| "".to_string());
    let mut user_email = use_signal(|| "".to_string());
    let mut user_pfp = use_signal(|| "".to_string());
    let mut refetch_signal = use_signal(|| false);
    let mut refetch_title = use_context::<Signal<bool>>();

    use_effect(move || {
        let mut _a = refetch_signal.read();
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
                            let current_refresh = *refetch_signal.read();
                            refetch_signal.set(!current_refresh);
                            // add some actuall error handeling
                            let mut current = !*refetch_title.read();
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