use dioxus::prelude::*;

#[component]
pub fn VideoPage(id: i64) -> Element {
    let navigator = use_navigator();
    let src = format!("https://www.vidking.net/embed/movie/{}?color=9146ff", id);

    rsx! {
        div {
            style: "padding: 20px; background-color: rgb(30, 30, 30); min-height: 100vh;",
            div {
                style: "margin-bottom: 20px;",
                button {
                    style: "background-color: rgb(147, 78, 213); color: white; border: none; padding: 10px 20px; border-radius: 6px; font-weight: 900; cursor: pointer;",
                    onclick: move |_| {
                        navigator.go_back();
                    },
                    "Back"
                }
            }
            iframe {
                src: "{src}",
                width: "100%",
                height: "600px",
            }
        }
    }
}
