use crate::frontend::*;
use crate::frontend::home_page::HomePage;
use crate::frontend::details::Details;
use crate::frontend::list_page::ListPgFn;
use crate::frontend::lists_page::ListsPgFn;
use crate::frontend::manage_user_profile::ManageAccount;
// fn names always have to have their first letter capital

#[derive(Routable, Clone)]
pub enum routes {
    #[layout(TitleBar)]
        #[route("/")]
        HomePage { },

        #[route("/details/:id")]
        Details { id: i64 },

        #[route("/list/:list_id/:user_id")]
        ListPgFn{ list_id: i64, user_id: i64 },

        #[route("/all_lists/:user_id")]
        ListsPgFn{ user_id: i64},

        #[route("/manage_account")]
        ManageAccount{ }
}