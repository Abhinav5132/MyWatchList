use dioxus::{desktop::{Config, WindowBuilder}, html::img::{alt, src}, prelude::*};
use serde_json::json;
use crate::router::routes;
mod search_page;
mod details;
mod router;
mod login_popup;

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

//add the token here as a GlobalSignal 
//when the token expires se this to null and ask the user to relogin
//make sure this token persists over app closures cuz if it dosent this token will have to be stored as a cookie ect somewhere
static TOKEN: GlobalSignal<String> = Signal::global(|| "".to_string());

pub fn main() {
    dioxus::LaunchBuilder::new().with_cfg(Config::default().with_menu(None)
    .with_window(
        WindowBuilder::new().with_maximized(true)
        .with_title("MyWatchList")
    )
).launch(App);
}

fn App() -> Element{
    rsx! { 
        document::Link{rel: "stylesheet", href: SEARCH_CSS}
        document::Link{rel: "stylesheet", href: LOGIN_CSS}
        document::Link{rel: "stylesheet", href: DETAILS_CSS}
        Router::<routes> {} 
    }
}