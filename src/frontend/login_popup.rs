use std::time;

use reqwest::Client;
use serde::Serialize;
pub use crate::frontend::*;

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
// everything here needs to be changed from user_id to user_token 

#[derive(Serialize)]
pub struct CheckUserNameAvailability{
    username: String,
}

#[derive(Deserialize)]
pub struct CheckUserNameAvailabilityResponse{
    available: bool
}

#[derive(Serialize)]
pub struct CheckEmailAvailability{
    email: String,
}

#[derive(Deserialize)]
pub struct CheckEmailAvailabilityResponse{
    available: bool
}

#[component] // it states username and email unavailable when typing one character at a time and disappears change this behaviour and change it from a p to a icon.
pub fn Login(on_close: EventHandler<()>)-> Element{
    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut password_again = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut trying_to_sign_up = use_signal(|| false);
    let mut username_available = use_signal(|| false);
    let mut email_available = use_signal(|| false);
    let mut debounce = use_signal(|| None::<std::time::Instant>);
    let mut debounce_email = use_signal(|| None::<std::time::Instant>);

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

                        if *trying_to_sign_up.read() {
                            username_available.set(false);

                            let client = Client::builder()
                                .build()
                                .expect("Failed to build client");
                            debounce.set(Some(time::Instant::now()));
                            spawn(async move{
                                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                                if let Some(last) = *debounce.read(){
                                    if last.elapsed().as_millis() >= 300 {
                                        let usrname = username.read();
                                        if let Ok(res) = client.get("http://localhost:3000/check_username_availability").json(
                                            &CheckUserNameAvailability{
                                                username: usrname.to_string()
                                            }
                                        )
                                        .send().await {
                                            let available = match res.json::<CheckUserNameAvailabilityResponse>().await {
                                                Ok(avail) => avail,
                                                Err(e) => {
                                                    dbg!(e);
                                                    CheckUserNameAvailabilityResponse{
                                                        available:false
                                                    } // sets user name as not available if unable to verify that username is available
                                                }
                                            };
                                            username_available.set(available.available);
                                            dbg!(available.available);
                                        }
                                    }
                                }
                            }); 
                        }
                    },
                    onkeydown: move |event| async move{ 
                        if event.code().to_string() == "Enter".to_string(){
                            if *trying_to_sign_up.read(){
                                let _ = document::eval(r#"document.getElementById('Login_email').focus();"#).await.unwrap();
                            } else{
                                let _ = document::eval(r#"document.getElementById('Login_password').focus();"#).await.unwrap();
                            }
                        }
                    },

                },
                if *username.read() != "".to_string() {
                    if !*username_available.read() {
                        p { 
                            "Username Unavailable. Please choose a different username."
                        }
                    } else{
                        p {  }
                    }
                } 

                if *trying_to_sign_up.read() {
                    label { "email:" },
                    input { 
                        
                        id:"Login_email",
                        type: "text",
                        oninput: move |event| {
                            event.prevent_default();
                            email.set(event.value());
                            email_available.set(false);

                            let client = Client::builder()
                                .build()
                                .expect("Failed to build client");
                            debounce_email.set(Some(time::Instant::now()));
                            spawn(async move{
                                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                                if let Some(last) = *debounce_email.read(){
                                    if last.elapsed().as_millis() >= 300 {
                                        let email = email.read();
                                        if let Ok(res) = client.get("http://localhost:3000/check_email_availability").json(
                                            &CheckEmailAvailability{
                                                email: email.to_string()
                                            }
                                        )
                                        .send().await {
                                            let available = match res.json::<CheckEmailAvailabilityResponse>().await {
                                                Ok(avail) => avail,
                                                Err(e) => {
                                                    dbg!(e);
                                                    CheckEmailAvailabilityResponse{
                                                        available:false
                                                    } // sets user name as not available if unable to verify that username is available
                                                }
                                            };
                                            username_available.set(available.available);
                                            dbg!(available.available);
                                        }
                                    }
                                }
                            }); 
                        },
                        onkeydown: move |event| async move{ 
                            if event.code().to_string() == "Enter".to_string(){
                                let _ = document::eval(r#"document.getElementById('Login_password').focus();"#).await.unwrap();
                            }
                        }
                    }

                    if *email.read() != "".to_string() {
                        if !*email_available.read() {
                                p { 
                                    "Email already taken please choose another email."
                                }
                            }
                        } else {
                            p {  }
                        }
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
                }

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
                            let client = Client::builder()
                            .danger_accept_invalid_certs(true)
                            .build()
                            .expect("Failed to build client");
                            spawn(async move{
                                // add actuall username and password checks

                                if let Ok(res) = client.post("http://localhost:3000/login").json(&LoginStruct{
                                    username: username.read().to_string(),
                                    password: password.read().to_string()
                                }).send().await{
                                    if let Some(auth_header) = res.headers().get("Authorization") {
                                        if let Ok(token_str) = auth_header.to_str(){
                                            let token = token_str.strip_prefix("Bearer ").unwrap_or(token_str);
                                            *TOKEN.write() = token.to_string(); // sets the token as a global signal that can be access anywhere 
                                            get_userid_from_jwt(); // gets the user id and stores in the global signal
                                            let path = storage_file();
                                            match fs::write(path, &token.to_string()){
                                                Ok(a)=> {
                                                    print!("Successfull wrote the token to");
                                                    a
                                                    
                                                }
                                                Err(e)=>{
                                                    dbg!("Failed to write token to the disk");
                                                    dbg!(e);
                                                }
                                            }
                                            print!("{token}");
                                            on_close.call(());
                                        }
                                    }
                                }
                            });
                        }
                        if *trying_to_sign_up.read(){
                            
                                let client = Client::builder()
                                .danger_accept_invalid_certs(true)
                                .build()
                                .expect("Failed to build client");
                                spawn(async move {
                                    if let Ok(res) = client.post("http://localhost:3000/Signup").json(&SignUpStruct{
                                        user_name: username.read().to_string(),
                                        user_email: email.read().to_string(),
                                        user_password: password.read().to_string()
                                    }).send().await {
                                        if let Some(auth_header) = res.headers().get("Authorization"){
                                            if let Ok(toker_str) = auth_header.to_str(){
                                                if *username_available.read() && *email_available.read(){
                                                    let token = toker_str.strip_prefix("Bearer ").unwrap_or(toker_str);
                                                    *TOKEN.write() = token.to_string();
                                                    get_userid_from_jwt();
                                                    let path = storage_file();
                                                    match fs::write(path, &token.to_string()){
                                                        Ok(a)=> {
                                                            a
                                                        }
                                                        Err(e)=>{
                                                            dbg!("Failed to write token to the disk");
                                                            dbg!(e);
                                                        }
                                                    }
                                                    on_close.call(());
                                                    print!("{token}");
                                                }

                                                else {
                                                    // show the cross again or make it grow in size and shrink back
                                                }
                                            }
                                        }
                                    }
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
