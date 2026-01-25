#![allow(clippy::redundant_field_names)]
use base64::{Engine as _, engine::general_purpose};
use dioxus::{
    desktop::{Config, WindowBuilder},
    prelude::*,
};
use reqwest::Client;
use serde::*;
use serde_json::Value;
use tokio::sync::Notify;
use std::{fs, path::PathBuf, sync::Arc, time::Duration};

mod details;
mod home_page;
mod router;
use router::routes;

use crate::frontend::{
    login_popup::{AuthResponse, get_refresh_token},
    title_bar::TitleBar,
};
pub mod first_time_page;
pub mod friends_page;
pub mod list_page;
pub mod lists_page;
pub mod logedin_dropdown;
mod login_popup;
pub mod manage_user_profile;
mod popup_add_anime;
pub mod popup_edit_list;
pub mod title_bar;
const LOGIN_CSS: Asset = asset!("/src/frontend/stylesheets/login_page.css");
const DETAILS_CSS: Asset = asset!("/src/frontend/stylesheets/details_page.css");
const SEARCH_CSS: Asset = asset!("/src/frontend/stylesheets/search_page.css");
const LIST_CSS: Asset = asset!("/src/frontend/stylesheets/lists_page.css");
const WELCOME_CSS: Asset = asset!("/src/frontend/stylesheets/welcome_page.css");

pub const HEART: Asset = asset!("assets/heart.png");
pub const TRAHSH: Asset = asset!("assets/bin.png");
pub const TICK: Asset = asset!("assets/check-mark.png");
pub const NOPFP: Asset = asset!("assets/No_pfp.jpg");
pub const ADD: Asset = asset!("assets/plus.png");
pub const PREV: Asset = asset!("assets/prev-page.png");
pub const NEXT: Asset = asset!("assets/next-page.png");
pub const MENU: Asset = asset!("assets/menu.png");
pub const PLAYLIST: Asset = asset!("assets/playlist.png");
pub const FRIENDS: Asset = asset!("assets/friends.png");
pub const THREEDOTS: Asset = asset!("assets/three_dots.jpg");
pub const XICON: Asset = asset!("assets/close.png");
pub const ADDFRIEND: Asset = asset!("assets/add.png");

fn storage_file() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("MyWatchList");
    fs::create_dir_all(&path).unwrap();
    path.push("username.json");
    path
}

#[derive(Serialize, Deserialize, Default)]
pub struct IssueNewAccess {
    access_token: String,
    expiry: u64,
}

// if the acess token is empty request for a new one. This occurs when the app is reopened
static TOKEN: GlobalSignal<String> = Signal::global(|| "".to_string());
static USERID: GlobalSignal<i64> = Signal::global(|| -1);
static REFRESHIN: GlobalSignal<i64> = Signal::global(|| -1);
static USERNAME: GlobalSignal<String> = Signal::global(|| "".to_string());
#[component]
pub fn App() -> Element {
    // if issue somewhere then just remove the username and attempt to remove the entry in the keyring
    let username = fs::read_to_string(storage_file()).unwrap_or_else(|_| "".to_string());
    let initial_username = use_signal(|| username);

    use_future(move || async move {
        let token_val = initial_username.read().clone();
        if !token_val.is_empty()
            && let Some(refresh_token) = get_refresh_token(&token_val)
        //TODO:  get_refresh_token needs to return a result
        {
            let issue_new_token = get_access_token(refresh_token).await;
            *TOKEN.write() = issue_new_token.access_token;
            *REFRESHIN.write() = issue_new_token.expiry as i64;
            *USERNAME.write() = token_val;
            get_userid_from_jwt();
            spawn_token_refreser();
        }
    });
    rsx! {
        document::Link{rel: "stylesheet", href: SEARCH_CSS}
        document::Link{rel: "stylesheet", href: LOGIN_CSS}
        document::Link{rel: "stylesheet", href: DETAILS_CSS}
        document::Link{rel: "stylesheet", href: LIST_CSS}
        document::Link{rel: "stylesheet", href: WELCOME_CSS}

        Router::<routes> { }
    }
}

pub fn launch_frontend() {
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::default().with_menu(None).with_window(
                WindowBuilder::new()
                    .with_maximized(true)
                    .with_title("MyWatchList"),
            ),
        )
        .launch(App);
}

pub async fn get_access_token(refresh_token: String) -> IssueNewAccess {
    let client = Client::new();
    if let Ok(res) = client
        .post("http://localhost:3000/issue_new_access")
        .json(&AuthResponse {
            access_token: "".to_string(),
            refresh_token: refresh_token,
            expires_in: 0,
        })
        .send()
        .await
        && let Ok(access_token) = res.json::<IssueNewAccess>().await
    {
        return access_token;
    }
    IssueNewAccess::default()
}

pub fn get_userid_from_jwt() {
    let token = TOKEN.read().clone();

    if !token.is_empty() {
        let base64_part = match token.split(".").nth(1) {
            Some(part) => part,
            None => {
                dbg!("Failed to find the b64 part");
                return;
            }
        };

        let bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(base64_part)
            .expect("Failed to decode base64");
        let join_str = match str::from_utf8(&bytes) {
            Ok(id) => id,
            Err(_e) => {
                dbg!("Failed to decode userid ");
                return;
            }
        };

        let json: Value = match serde_json::from_str(join_str) {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Failed to parse JWT payload as JSON.");
                return;
            }
        };

        let userid = match json.get("sub").and_then(Value::as_i64) {
            Some(id) => id,
            None => {
                eprintln!("No user_id field in JWT payload.");
                return;
            }
        };
        *USERID.write() = userid
    }
}

pub fn spawn_token_refreser() {
    let user_id = USERNAME.read().to_string();
    spawn(async move {
        loop {
            if *REFRESHIN.read() != -1 {
                let expiry = (*REFRESHIN.read() - 60) as u64;
                let now = chrono::Utc::now().timestamp() as u64;

                let wait_secs = expiry.saturating_sub(now + 60);
                let wait_time = Duration::from_secs(wait_secs);

                tokio::time::sleep(wait_time).await;

                if let Some(refresh_token) = get_refresh_token(&user_id) {
                    match get_access_token(refresh_token).await {
                        new_token if !new_token.access_token.is_empty() => {
                            *TOKEN.write() = new_token.access_token;
                            *REFRESHIN.write() = new_token.expiry as i64;
                            get_userid_from_jwt();
                            dbg!("Token refreshed successfully");
                            dbg!(TOKEN.read());
                        }
                        _ => {
                            dbg!("Failed to refresh token, retrying in 30s...");
                            tokio::time::sleep(Duration::from_secs(30)).await;
                        }
                    }
                } else {
                    //TODO: THIS WHOLE THING NEEDS TO BE REDONE. (AUTH REWORK COMING SOON)
                    dbg!("No refresh token found, stopping refresher");
                    let path = storage_file();
                    match fs::write(path, "{}") {
                        Ok(a) => {
                            print!("Successfull wrote the token to");
                            a
                        }
                        Err(e) => {
                            dbg!("Failed to write token to the disk");
                            dbg!(e);
                        }
                    }
                    break; // empty the username store here
                }
            } else {
                break;
            }
        }
    });
}
