use crate::frontend::details::Details;
use crate::frontend::first_time_page::FirstTimePage;
use crate::frontend::friends_page::FriendPage;
use crate::frontend::home_page::HomePage;
use crate::frontend::list_page::ListPgFn;
use crate::frontend::lists_page::ListsPgFn;
use crate::frontend::manage_user_profile::ManageAccount;
use crate::frontend::*;
// fn names always have to have their first letter capital

#[derive(Routable, Clone)]
pub enum routes {
    #[route("/")]
    FirstTimePage {},
    #[layout(TitleBar)]
    #[route("/home")]
    HomePage {},

    #[route("/details/:id")]
    Details { id: i64 },

    #[route("/list/:list_id/:user_id")]
    ListPgFn { list_id: i64, user_id: i64 },

    #[route("/all_lists/:user_id")]
    ListsPgFn { user_id: i64 },

    #[route("/manage_account")]
    ManageAccount {},

    #[route("/friends_page")]
    FriendPage {},
}
