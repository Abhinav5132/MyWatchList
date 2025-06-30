use dioxus::{html::{img::src, img::alt}, prelude::*};
use dioxus_router::prelude;

use crate::router::routes;
mod search_page;
mod details;
mod router;

pub fn main() {
    dioxus::launch(App);
}

fn App() -> Element{
    rsx! { Router::<routes> {} }
}