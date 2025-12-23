pub use crate::frontend::*;
pub use dioxus::prelude::spawn;
#[derive(Serialize, Deserialize, Default)]
pub struct UserDetails {
    username: String,
    user_email: String,
    user_pfp: String,
}

#[component]
pub fn loged_in_dropdown(
    username: String,
    user_email: String,
    onclose: EventHandler<()>,
    user_image: String,
) -> Element {
    let navigator = use_navigator();
    rsx!(
        div {
            id: "user_logedin_dropdown",
            div {
                class: "dropdown-item user-info",
                img {
                    id: "Dropdown_user_image",
                    src:"{user_image}",
                }
                h2 { "{username}" }
                h4 { "{user_email}" }
            }

            div {
                class: "dropdown-item",
                onclick: move |_| {
                    navigator.push(crate::frontend::router::routes::ManageAccount {  });
                },
                "Manage account"
            }

            div {
                class: "dropdown-item",
                onclick: move |_| {
                    navigator.push(crate::frontend::router::routes::ListsPgFn { user_id: *USERID.read() });
                },
                "Manage your lists"
            }

            div {
                class: "dropdown-item logout",
                onclick: move |_| {
                    logout();
                    // this part of the code just dosent work for some reasonp
                    spawn(async move {
                        dbg!("async task started");
                        let token = TOKEN.read().clone();
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

pub fn logout() {
    let username = USERNAME.read().clone();

    dbg!("Logging out");
    if let Err(e) = match keyring::Entry::new("MyWatchList", username.as_str()) {
        Ok(a) => a.delete_credential(),
        Err(e) => Err(e),
    } {
        dbg!(e);
    }
    *USERNAME.write() = "".to_string();
    *USERID.write() = -1;
    *TOKEN.write() = "".to_string();
    *REFRESHIN.write() = -1;

    if let Err(e) = std::fs::remove_file(storage_file())
        && e.kind() != std::io::ErrorKind::NotFound
    {
        eprintln!("Failed to remove storage file: {:?}", e);
    }
}
