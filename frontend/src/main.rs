use dioxus::{desktop::{Config, WindowBuilder}, html::img::{alt, src}, prelude::*};

use crate::router::routes;
mod search_page;
mod details;
mod router;
mod login_page;
mod sign_up_page;
const LOGIN_CSS:Asset = asset!("/stylesheets/login_page.css");
const DETAILS_CSS: Asset = asset!("/stylesheets/details_page.css");
const SEARCH_CSS: Asset = asset!("/stylesheets/search_page.css");

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