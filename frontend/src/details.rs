use crate::*;

const DETAILS_CSS: Asset = asset!("/assets/details_page.css");

#[component]
pub fn Details(id: i32) -> Element{
    rsx!{
        h1 {"This is the details page "}
    }
}