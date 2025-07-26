use dioxus::{desktop::{tao::window, use_window, window}, html::img::{alt, src}, prelude::*};
use dioxus_router::prelude::*;
use dioxus_core::Event;

const LOGIN_CSS:Asset = asset!("/stylesheets/login_page.css");

#[component]
pub fn Login()-> Element{
    let mut username_email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    rsx!(
        document::Link{rel: "stylesheet", href: LOGIN_CSS },
        div { 
            id:"Login_class",
            
            label { "Email/Username:" }
            input { 
                
                id:"Login_email",
                r#type: "text",
                oninput: move |event| {
                    username_email.set(event.value());
                },
                onkeydown: move |event| async move{ 
                    if event.code().to_string() == "Enter".to_string(){
                        let _ = document::eval(r#"document.getElementById('Login_password').focus();"#).await.unwrap(); // actually deal with this shit in prod
                    }
                }
            }
            label { "Password:" }
            input { 
                id:"Login_password",
                r#type: "password",
                oninput: move |event| {
                    password.set(event.value());
                },
                onkeydown: move |event| async move {
                    if event.code().to_string() == "Enter".to_string(){
                       let _ = document::eval(r#"document.getElementById('submit_button').click();"#).await.unwrap();
                    }
                }
            }

            button {  
                id: "submit_button",
                r#type:"button",
                onclick: move |_|{
                    println!("Button clicked")   
                },
                "Submit"
                
            }

        }
    )
}