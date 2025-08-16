
pub use crate::*;
#[component]
pub fn PopupAddAnime(is_error: bool, list_name: String, anime_name: String, on_close: EventHandler<()>) -> Element {
    rsx!(
        div{
            id: "popup_anime_div",
            if is_error{
                p { "Unable to add {anime_name} to {list_name}. Please Try again." },

                button { 
                    id:"Try_again_popup",
                    onclick: move |_| {

                    },
                    "Try Again"
                }
                button { 
                    class:"close_popup_button",
                    onclick: move |_| {
                        on_close.call(());
                    },
                    "Close"
                }
                
            } else{
                p { "Succesfully added {anime_name} to {list_name}." }

            }


        }
    )
}