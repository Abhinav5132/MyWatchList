use dioxus::{desktop::{Config, WindowBuilder}, html::img::{alt, src}, prelude::*};

use crate::router::routes;
mod search_page;
mod details;
mod router;
mod login_page;
mod sign_up_page;

pub fn main() {
    dioxus::LaunchBuilder::new().with_cfg(Config::default().with_menu(None)
    .with_window(
        WindowBuilder::new().with_maximized(true)
        .with_title("MyWatchList")
    )
).launch(App);
}

fn App() -> Element{
    rsx! { Router::<routes> {} }
}