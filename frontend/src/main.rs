

use dioxus::{desktop::{Config, WindowBuilder}, html::img::{alt, src}, prelude::*};
use base64::{Engine as _, alphabet, engine::{self, general_purpose}};
use serde::*;
use serde_json::Value;
use crate::router::routes;
mod search_page;
mod details;
mod router;
mod login_popup;
mod popup_add_anime;

const LOGIN_CSS:Asset = asset!("/stylesheets/login_page.css");
const DETAILS_CSS: Asset = asset!("/stylesheets/details_page.css");
const SEARCH_CSS: Asset = asset!("/stylesheets/search_page.css");

pub const HEART:Asset = asset!("/assets/heart.png");
pub const TRAHSH:Asset = asset!("/assets/bin.png");
pub const TICK:Asset = asset!("/assets/check-mark.png");
pub const NOPFP:Asset = asset!("/assets/No_pfp.jpg");
pub const ADD:Asset = asset!("/assets/plus.png");
pub const PREV: Asset = asset!("/assets/prev-page.png");
pub const NEXT: Asset = asset!("/assets/next-page.png");
pub const MENU: Asset = asset!("/assets/menu.png");
pub const PLAYLIST: Asset = asset!("/assets/playlist.png");
pub const FRIENDS: Asset = asset!("/assets/friends.png");


#[derive(Serialize, Deserialize)]
pub struct Claims{
    pub sub: i64,
    pub exp: usize
}

//add the token here as a GlobalSignal 
//when the token expires se this to null and ask the user to relogin
//make sure this token persists over app closures cuz if it dosent this token will have to be stored as a cookie ect somewhere
static TOKEN: GlobalSignal<String> = Signal::global(|| "".to_string());
static USERID: GlobalSignal<i64> = Signal::global(|| -1);

pub fn main() {
    // find a way to store the token somewhere that isnt a globalsignal
    dioxus::LaunchBuilder::new().with_cfg(Config::default().with_menu(None)
    .with_window(
        WindowBuilder::new().with_maximized(true)
        .with_title("MyWatchList")
        )
    ).launch(App);
    get_userid_from_jwt();
}

fn App() -> Element{
    rsx! { 
        document::Link{rel: "stylesheet", href: SEARCH_CSS}
        document::Link{rel: "stylesheet", href: LOGIN_CSS}
        document::Link{rel: "stylesheet", href: DETAILS_CSS}
        Router::<routes> {} 
    }
}

pub fn get_userid_from_jwt() {
    let token = TOKEN.read().clone();

    if token != "".to_string(){
        let base64_part = match token.split(".").nth(1) {
            Some(part)=> part,
            None => {
                dbg!("Failed to find the b64 part")
            }
        };

        let bytes = general_purpose::URL_SAFE_NO_PAD.decode(base64_part).expect("Failed to decode base64");
        let join_str = match str::from_utf8(&bytes){
            Ok(id)=>id,
            Err(e)=>{
                dbg!("Failed to decode userid ")
            }
        };

        let json:Value = match serde_json::from_str(join_str) {
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