pub use crate::frontend::*;

#[derive(Serialize, Deserialize, Default)]
pub struct UserDetails{
    username: String,
    user_email: String,
    user_pfp: String,
}

#[component]
pub fn loged_in_dropdown(username: String, user_email: String, onclose: EventHandler<()>)-> Element{

    rsx!(
        div { 
            id: "user_logedin_dropdown",
            div {
                class: "dropdown-item user-info",
                h2 { "{username}" }
                h4 { "{user_email}" }
            }

            div {
                class: "dropdown-item",
                onclick: move |_| {
                    // TODO: route to manage account
                },
                "Manage account"
            }

            div {
                class: "dropdown-item",
                onclick: move |_| {
                    // TODO: route to lists page
                },
                "Manage your lists"
            }

            div {
                class: "dropdown-item logout",
                onclick: move |_| {
                    spawn(async move {
                        logout().await;
                    });
                    
                    onclose.call(()); // also close dropdown after logout
                },
                "Log out"
            }

            div {
                class: "dropdown-item close",
                onclick: move |_| {
                    onclose.call(());
                },
                "Close"
            }
        }
    )
}


pub async fn logout() {
    let username = USERNAME.read().clone();
    let token = TOKEN.read().clone();
    dbg!("Logging out");
     if let Err(e) = match keyring::Entry::new("MyWatchList", username.as_str()) {
        Ok(a) => a.delete_credential(),
        Err(e) => {
            Err(e)
        }
    } {
        dbg!(e);
    }
    
    let client = Client::new();
    match client.post("http://localhost:3000/logout")
    .bearer_auth(token)
    .send().await {
        Ok(_) => {
            dbg!("Logout request sent");
        } 
        Err(e) => {
            dbg!(e);
        }
    }


    *USERNAME.write() = "".to_string();
    *USERID.write() = -1;
    *TOKEN.write() = "".to_string();
    *REFRESHIN.write() = -1;

    if let Err(e) = std::fs::remove_file(storage_file()) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!("Failed to remove storage file: {:?}", e);
        }
    }

   
}