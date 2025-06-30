use crate::*;
use crate::search_page::Searchpg;
use crate::details::Details;
#[derive(Routable, Clone)]
pub enum routes {
    #[route("/")]
    Searchpg {},

    #[route("/details/:id")]
    Details { id: i32 },
}