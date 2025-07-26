use crate::*;
use crate::search_page::Searchpg;
use crate::details::Details;
use crate::login_page::Login;

// fn names always have to have their first letter capital

#[derive(Routable, Clone)]
pub enum routes {
    #[route("/search")]
    Searchpg {},

    #[route("/details/:id")]
    Details { id: i32 },

    #[route("/")]
    Login {},
}