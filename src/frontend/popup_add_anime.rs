
pub use crate::frontend::*;
#[component]
pub fn PopupAddAnime( 
    list_name: String,
    anime_name: String, 
    is_rank: bool, 
    last_rank: i32, 
    on_close: EventHandler<()>, 
    on_submit: EventHandler<i32>) -> Element {
    let mut new_selected_rank = use_signal(|| last_rank + 1);
    rsx!{
        if is_rank {
            div { 
                id: "is_ranked_popup",
                class: "is_ranked_popup",
                p {"Please rank the entry: {anime_name}"},
                input { 
                    type:"number",
                    name:"Rank",
                    min:"1",
                    max:last_rank + 1,
                    value: *new_selected_rank.read(),
                    oninput: move |event| {
                        let val:i32 = event.value().parse().unwrap_or(-1); // -1 for now
                        new_selected_rank.set(val);
                    }

                }

                button { 
                    id:"submit_ranked_popup",
                    onclick: move |_|{
                        on_close.call(());
                        on_submit.call(*new_selected_rank.read());
                        },
                    "Submit"   
                    }
                button { 
                    id:"close_ranked_popup",
                    onclick: move |_|{
                        on_close.call(());
                        },
                        "Cancel"
                    }
                    
                }
                    
            } 
        else {
            div { 
                class: "is_ranked_popup",
                id: "is_unranked_popup",
                p { "{ anime_name } added successfully to { list_name}" } 

                button { 
                    id:"close_ranked_popup",
                    onclick: move |_| {
                        on_close.call(());
                        on_submit.call(*new_selected_rank.read());
                    },
                    "Close"
                }                      
            }
        }

    }
}

#[component]
pub fn PopupError(list_name: String,
    anime_name: String,on_close: EventHandler<()>) -> Element{
    rsx!(
    div{
        id: "popup_anime_div",
            p { "Unable to add {anime_name} to {list_name}. Please Try again." },
            div {  
                id:"popup_buttons_div",
                button { 
                    id:"Try_again_popup",
                    onclick: move |_| {
                        //does nothing for now
                    },
                    "Try Again"
                },
                button { 
                    class:"close_popup_button",
                    onclick: move |_| {
                        on_close.call(());
                    },
                    "Close"
                }
            } 
            } 
    )
}