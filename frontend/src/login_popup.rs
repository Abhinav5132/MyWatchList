use dioxus::{prelude::*};
use reqwest::Client;
use serde::Serialize;

use crate::TOKEN;

#[derive(Serialize)]
pub struct LoginStruct {
    username: String,
    password: String
}
#[component]
pub fn Login(on_close: EventHandler<()>)-> Element{
    let mut username_email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let navigator = use_navigator();

    
    rsx!(
        div{ 
            id: "Main_div",
            div { 
                id:"Login_class",
                h3 { 
                    id:"title_login",
                    "MyWatchList" 
                },
                label { "Email/Username:" },
                input { 
                    
                    id:"Login_email",
                    r#type: "text",
                    oninput: move |event| {
                        event.prevent_default();
                        username_email.set(event.value());
                    },
                    onkeydown: move |event| async move{ 
                        if event.code().to_string() == "Enter".to_string(){
                            let _ = document::eval(r#"document.getElementById('Login_password').focus();"#).await.unwrap(); // actually deal with this shit in prod
                        }
                    }
                },
                label { "Password:" },
                input { 
                    id:"Login_password",
                    r#type: "password",
                    oninput: move |event| {
                        event.prevent_default();
                        password.set(event.value());
                    },
                    onkeydown: move |event| async move {
                        if event.code().to_string() == "Enter".to_string(){
                        let _ = document::eval(r#"document.getElementById('submit_button').click();"#).await.unwrap();
                        }
                    }
                },

                button {  
                    id: "submit_button",
                    r#type:"button",
                    onclick: move |_| {
                         // make sure this only gets called if login is actually succesfull
                         use_effect(move || {
                            let client = use_context::<Client>();
                            spawn(async move{
                                // add actuall username and password checks
                                if let Ok(res) = client.post("/login").json(&LoginStruct{
                                    username: username_email.read().to_string(),
                                    password: password.read().to_string()
                                }).send().await{
                                    if let Some(auth_header) = res.headers().get("Authorization") {
                                        if let Ok(token_str) = auth_header.to_str(){
                                            let token = token_str.strip_prefix("Bearer ").unwrap_or(token_str);
                                            *TOKEN.write() = token.to_string(); // sets the token as a global signal that can be access anywhere 
                                            print!("{token}");
                                            on_close.call(());
                                        }
                                    }
                                }
                            });
                            ()
                        });
                    },
                    "Submit"
                }
                p { 
                    "Not a member " 
                    a { 
                        class: "link_text",
                        onclick: move |_|{
                            navigator.push(crate::router::routes::Searchpg {  });
                        },
                        "sign up"
                    }
                    " or "
                    a { 
                        class: "link_text",
                        onclick: move |_|{
                            navigator.push(crate::router::routes::Searchpg {  });
                        },
                        "continue as guest."
                    }
                }
            }    
        }    
    )
}