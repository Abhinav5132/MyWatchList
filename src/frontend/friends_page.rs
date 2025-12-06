use dioxus::html::script::r#async;

use crate::frontend::lists_page::AList;
pub use crate::frontend::*;
#[derive(Deserialize)]
pub struct FriendRequest {
    user_id: i64,
    friend_id: i64, 
}

#[derive(Deserialize)]
pub struct RequestId {
    request_id: i64
}

#[derive(Deserialize)]
pub struct FriendId {
    friend_id: i64
}

#[derive(Deserialize, Serialize)]
pub struct Friend {
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
#[component]
pub fn FriendPage() -> Element {
    let mut all_friends = use_signal(|| vec![]);
    use_future( move || {
        let client = Client::new();
        async move {
            if let Ok(req) = client.get("http://localhost:3000/get_all_friends")
            .bearer_auth(TOKEN.read()).send().await {
                if let Ok(Afriends) = req.json::<AllFriends>().await{
                    all_friends.set(Afriends.friends);
                }
            }
    }});

    rsx!(
        div { 
            

        }
    ) // TODO friends page 
}