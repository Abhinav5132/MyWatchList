use std::sync::Arc;

use crate::global_db::get_db_pool;
use dioxus::{html::title, prelude::*};
use serde_json::to_string;
use sqlx::{sqlite::SqliteRow, Pool, Sqlite};

const MAIN_CSS: Asset = asset!("/assets/main.css");

pub fn search_main() {
    dioxus::launch(App);
}

 struct Anime {
        title: String,
    }

#[component]
pub fn App() -> Element {
    let conn = get_db_pool();

   
    let mut search_input = use_signal(|| "".to_string());
    let mut submitted_title = use_signal(|| "Please enter anime name here".to_string());
    rsx! {
        div {
            document::Link{rel: "stylesheet", href: MAIN_CSS}
            h1 { "Search for an anime" }
            div {
                input {
                    id: "Search_Bar",
                    type: "search",
                    value: "{search_input}",
                    oninput: move |event| {
                        search_input.set(event.value());
                    },
                    onkeydown: move |event| {
                        if event.code().to_string() == "ENTER".to_string() {
                            submitted_title.set(search_input.read().clone()); }
                    }
                }
            button {
                id: "Search_buttone",
                onclick: move |_| {
                    submitted_title.set(search_input.read().clone());
                },
                "search"
                }
            }

        }

    }
}

async fn lookup_database(conn: Arc<Pool<Sqlite>>, mut title: String) -> Vec<SqliteRow>{

    title = format!("%{}%", title);

    let rows = sqlx::query("SELECT name FROM anime WHERE name like ?")
    .bind(title)
    .fetch_all(&*conn)
    .await;

    rows.unwrap()


}