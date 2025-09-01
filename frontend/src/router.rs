use crate::*;
use crate::search_page::Searchpg;
use crate::details::Details;
use crate::list_page::ListPgFn;
// fn names always have to have their first letter capital

#[derive(Routable, Clone)]
pub enum routes {
    #[route("/")]
    Searchpg {},

    #[route("/details/:id")]
    Details { id: i64 },

    #[route("/list/:list_name/:user_id")]
    ListPgFn{ list_name: String, user_id: i64 }
}