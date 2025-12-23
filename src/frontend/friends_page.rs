
use crate::frontend::lists_page::AList;
pub use crate::frontend::*;
#[derive(Deserialize)]
pub struct FriendRequest {
    user_id: i64,
    friend_id: i64, 
}

#[derive(Deserialize, Serialize)]
pub struct RequestId {
    request_id: i64
}

#[derive(Deserialize, Serialize)]
pub struct FriendId {
    friendship_id: i64
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Friend {
    friendship_id: i64,
    friend_id: i64,
    user_name: String,
    user_pfp: String,
}

#[derive(Deserialize, Serialize)]
pub struct AllFriends {
    friends: Vec<Friend>
}

#[derive(Deserialize, Serialize)]
pub struct FullFriend {
    friend_id: i64,
    user_name: String,
    user_pfp: String,
    lists: Vec<AList>
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum FriendRequestDirection {
    INCOMING,
    SENDING,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FriendRequests {
    friend_id: i64,
    user_name: String,
    user_pfp: String,
    direction: FriendRequestDirection,
    req_id: i64,
}

#[derive(Deserialize, Serialize)]
pub struct AllFriendRequests {
    friend_requests: Vec<FriendRequests>
}
#[component]
pub fn FriendPage() -> Element {
    let mut all_friends = use_signal(|| vec![]);
    let mut all_requests = use_signal(|| vec![]);
    let mut show_friends_dropdown = use_signal(|| false);
    let mut show_remove_friends_popup = use_signal(|| false);
    let mut refresh_page = use_signal(|| false);
    use_future( move || {
        let _ = refresh_page.read();
        let client = Client::new();
        async move {
            if let Ok(req) = client.get("http://localhost:3000/get_all_friends")
            .bearer_auth(TOKEN.read()).send().await
                && let Ok(Afriends) = req.json::<AllFriends>().await{
                    all_friends.set(Afriends.friends);
                }
    }});

    use_future(move ||{
        let _ = refresh_page.read();
        let client = Client::new();
        async move{
            if let Ok(req) = client.get("http://localhost:3000/get_all_friends_requests")
            .bearer_auth(TOKEN.read()).send().await
                && let Ok(friend_requests) = req.json::<AllFriendRequests>().await{
                    all_requests.set(friend_requests.friend_requests);
                }
        }
        }
    );

    rsx!(
        div { 
            id: "Containing_div",
            div { 
                id: "Friends_div",
                h3 { "Friends" },
                for friend in all_friends.read().clone(){
                    div { 
                        class: "Friend_card_div",
                        onclick: move |_| {
                            // on click should redirect to that persons profile page that shows of their public lists and friends only lists.
                        },
                        h4 { "{friend.user_name}" }
                        img { 
                            class: "Friend_profile_image",
                            src: friend.user_pfp,
                        }
                        button { // three dots more info buton
                            class: "more_info_button",
                            onclick: move |evt| {
                                evt.stop_propagation();
                                show_friends_dropdown.set(true);
                            },
                            img { 
                                class: "Feeling_icon",
                                src:THREEDOTS,
                            }
                            
                        }

                        if *show_friends_dropdown.read() {
                            div{
                                class: "friends_dropdown",
                                show_dropdown { 
                                    on_close: move |_|{
                                        show_friends_dropdown.set(false);
                                    }
                                }
                            }
                        }   
                        
                    }
                }

            }

            div { 
                id:"Pending_friends_list",
                h3 { "Pending Requests" }
                div { 
                    class: "request_type_div",
                    h4 { "Incoming" },
                    for req in all_requests.read().clone(){
                        if req.direction == FriendRequestDirection::INCOMING{
                            div { 
                                class: "Friend_card_div",
                                h4 { "req.user_name" }
                                img { 
                                    class:"Friend_profile_image",
                                    src: req.user_pfp
                                }
                                img { 
                                    class:"Feeling_icon",
                                    src: TICK,
                                    onclick: move |_| {
                                        spawn(async move {
                                            let client = Client::new();
                                            if let Ok(res) = client.post("http://localhost:3000/accept_friend_request").json(
                                                &RequestId{
                                                    request_id: req.req_id,
                                                }
                                            )
                                            .bearer_auth(TOKEN.read()).send().await{
                                                let negate = !*refresh_page.read(); // refreshing the page
                                                refresh_page.set(negate);
                                            }
                                        });
                                    }
                                }
                                img { 
                                    class:"Feeling_icon",
                                    src: XICON,
                                    onclick: move |_| {
                                        spawn(async move {
                                            let client = Client::new();
                                            if let Ok(res) = client.post("http://localhost:3000/decline_friend_request")
                                            .json(
                                                &RequestId{
                                                    request_id: req.req_id,
                                                })
                                            .bearer_auth(TOKEN.read()).send().await{
                                                let negate = !*refresh_page.read(); // refreshing the page
                                                refresh_page.set(negate);
                                                //TODO failed to execute message if server returns an error.
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                div { 
                    class: "request_type_div",
                    h4 { "Sent" },
                    for req in all_requests.read().clone(){
                        if req.direction == FriendRequestDirection::SENDING{
                            div { 
                                class: "Friend_card_div",
                                h4 { "req.user_name" }
                                img { 
                                    class:"Friend_profile_image",
                                    src: req.user_pfp
                                }
                                img { 
                                    class:"Feeling_icon",
                                    src: XICON,
                                    onclick: move |_| {
                                        spawn(async move {
                                            let client = Client::new();
                                            if let Ok(res) = client.post("http://localhost:3000/decline_friend_request")
                                            .json(
                                                &RequestId{
                                                    request_id: req.req_id,
                                                })
                                            .bearer_auth(TOKEN.read()).send().await{
                                                let negate = !*refresh_page.read(); // refreshing the page
                                                refresh_page.set(negate);
                                                //TODO failed to execute message if server returns an error.
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}

#[component]
pub fn show_dropdown(on_close: EventHandler)-> Element{
    rsx!(
        button { 
            class: "dropdown_button",
            id: "remove_friend",
            onclick: move |_| {
                // should use friendship_id to remove.
                on_close.call(());
            },
            "Remove Friend"
        }
        button { 
            class: "dropdown_button",
            onclick: move |_| {
                // redirect to friends profile
                on_close.call(());

            },
            "Profile"
        }
        button { 
            class: "dropdown_button",
            onclick: move |_| {
                // redirect to friends list page
                on_close.call(());

            },
            "Currently Watching"
        }
        button { 
            class: "dropdown_button",
            onclick: move |_| {
                // redirect to friends profile
                on_close.call(());

            },
            "Recommendations"
        }
    )
}