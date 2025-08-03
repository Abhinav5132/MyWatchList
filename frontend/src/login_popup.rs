use std::ops::Not;

use dioxus::{prelude::*};
use reqwest::Client;
use serde::Serialize;

use crate::TOKEN;

#[derive(Serialize)]
pub struct LoginStruct {
    username: String,
    password: String
}

#[derive(Serialize)]
pub struct SignUpStruct{
    user_name: String,
    user_password: String,
    user_email: String,

}

#[component]
pub fn Login(on_close: EventHandler<()>)-> Element{
    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut password_again = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let navigator = use_navigator();
    let mut trying_to_sign_up = use_signal(|| false);
    
    rsx!(
        div{ 
            id: "Main_div",
            div { 
                class:"Login_class", // i can use the multiple classes trick to make the scaling of the man div work
                h3 { 
                    id:"title_login",
                    "MyWatchList" 
                },
                label { "Username:" },
                input { 
                    
                    id:"Login_username",
                    r#type: "text",
                    oninput: move |event| {
                        event.prevent_default();
                        username.set(event.value());
                    },
                    onkeydown: move |event| async move{ 
                        if event.code().to_string() == "Enter".to_string(){
                            if *trying_to_sign_up.read(){
                                let _ = document::eval(r#"document.getElementById('Login_email').focus();"#).await.unwrap();
                            } else{
                                let _ = document::eval(r#"document.getElementById('Login_password').focus();"#).await.unwrap();
                            }
                        }
                    }
                },

                if *trying_to_sign_up.read() {
                    label { "email:" },
                    input { 
                        
                        id:"Login_email",
                        type: "text",
                        oninput: move |event| {
                            event.prevent_default();
                            email.set(event.value());
                        },
                        onkeydown: move |event| async move{ 
                            if event.code().to_string() == "Enter".to_string(){
                                let _ = document::eval(r#"document.getElementById('Login_password').focus();"#).await.unwrap();
                            }
                        }
                    },
    
                }
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
                            if *trying_to_sign_up.read() {
                                let _ = document::eval(r#"document.getElementById('Login_password_again').click();"#).await.unwrap();
                            }else{
                                let _ = document::eval(r#"document.getElementById('submit_button').click();"#).await.unwrap();
                            }
                        }
                    }
                },

                if *trying_to_sign_up.read() {
                    label { "re-enter password:" },
                    input { 
                        
                        id:"Login_password_again",
                        type: "text",
                        oninput: move |event| {
                            event.prevent_default();
                            email.set(event.value());
                        },
                        onkeydown: move |event| async move{ 
                            if event.code().to_string() == "Enter".to_string(){
                                let _ = document::eval(r#"document.getElementById('submit_button').focus();"#).await.unwrap();
                            }
                        }
                    },
    
                }

                button {  
                    id: "submit_button",
                    r#type:"button",
                    onclick: move |_| {
                        if !*trying_to_sign_up.read(){
                            use_effect(move || {
                                let client = Client::builder()
                                .danger_accept_invalid_certs(true)
                                .build()
                                .expect("Failed to build client");
                                spawn(async move{
                                    // add actuall username and password checks
                                    if let Ok(res) = client.post("https://localhost:3000/login").json(&LoginStruct{
                                        username: username.read().to_string(),
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
                        }
                        if *trying_to_sign_up.read(){
                            use_effect(move ||{
                                let client = Client::builder()
                                .danger_accept_invalid_certs(true)
                                .build()
                                .expect("Failed to build client");
                                spawn(async move {
                                    if let Ok(res) = client.post("https://localhost:3000/Signup").json(&SignUpStruct{
                                        user_name: username.read().to_string(),
                                        user_email: email.read().to_string(),
                                        user_password: password.read().to_string()
                                    }).send().await {
                                        if let Some(auth_header) = res.headers().get("Authorization"){
                                            if let Ok(toker_str) = auth_header.to_str(){
                                                let token = toker_str.strip_prefix("Bearer ").unwrap_or(toker_str);
                                                on_close.call(());
                                                print!("{token}");
                                            }
                                        }
                                    }
                                });
                                ()
                            });
                        }
                    },
                    "Submit"
                }
                p { 
                    "Not a member " 
                    a { 
                        class: "link_text",
                        onclick: move |_|{
                            trying_to_sign_up.set(true);
                        },
                        "sign up"
                    }
                    " or "
                    a { 
                        class: "link_text",
                        onclick: move |_|{
                            on_close.call(());
                        },
                        "continue as guest."
                    }
                }
            }    
        }    
    )
}
