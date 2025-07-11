use dioxus::{desktop::{Config, WindowBuilder}, html::img::{alt, src}, prelude::*};

use crate::router::routes;
mod search_page;
mod details;
mod router;

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