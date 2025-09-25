use crate::frontend::*;
use crate::frontend::search_page::Searchpg;
use crate::frontend::details::Details;
use crate::frontend::list_page::ListPgFn;
use crate::frontend::lists_page::ListsPgFn;
// fn names always have to have their first letter capital

#[derive(Routable, Clone)]
pub enum routes {
    #[route("/")]
    Searchpg {},

    #[route("/details/:id")]
    Details { id: i64 },

    #[route("/list/:list_name/:user_id")]
    ListPgFn{ list_name: String, user_id: i64 },

    #[route("/all_lists/:user_id")]
    ListsPgFn{ user_id: i64}
}