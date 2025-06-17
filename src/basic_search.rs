use std::sync::Arc;

use crate::global_db::get_db_pool;
use dioxus::{html::title, prelude::*};
use serde_json::to_string;
use sqlx::{sqlite::SqliteRow, Pool, Sqlite, Row};
use tokio::task::block_in_place;

const MAIN_CSS: Asset = asset!("/assets/main.css");

pub fn search_main() {
    dioxus::launch(App);
}

 struct Anime {
        title: String,
    }

#[component]
pub fn App() -> Element {

   
    let mut search_input = use_signal(|| "".to_string());
    let mut submitted_title = use_signal(|| "Please enter anime name here".to_string());
    let anime_results = use_resource(move || {
        // We need to clone the connection pool and the title to move them into the async block.
        let title = submitted_title.read().clone();
        async move {
            let conn = get_db_pool();
            lookup_database(conn, title).await
        }
    });
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
            div {
                class: "results",
                h2 { "Search Results" }
                // Here we handle all states of our resource: Loading, Success, and Error
                {
                    match &*anime_results.read() {
                        // The resource is still loading
                        None => rsx! { p { "Loading..." } },
                        // The resource has loaded successfully
                        Some(Ok(animes)) => {
                            let num = animes.len();
                            if num > 0 {
                                rsx! {
                                    p { "Found {num} results for '{submitted_title}'" }
                                    ul {
                                        {animes.iter().map(|row| {
                                            let title: String = row.get("title");
                                            rsx! { li { "{title}" } }
                                        })}
                                    }
                                }
                            } else {
                                // Don't show "Found 0" on initial load
                                if *submitted_title.read() != "Please enter anime name here" {
                                    rsx! { p { "No results found for '{submitted_title}'" } }
                                } else {
                                    rsx! { p { "Please enter an anime name to search." } }
                                }
                            }
                        },
                        // The resource failed to load
                        Some(Err(e)) => rsx! {
                            p { class: "error", "Error fetching data from database: {e}" }
                        }
                    }
                }
            }
        }

    }
}

async fn lookup_database(conn: Arc<Pool<Sqlite>>, mut title: String) -> Result<Vec<SqliteRow>, sqlx::Error>{

    title = format!("%{}%", title);

    let rows = sqlx::query("SELECT title FROM anime WHERE title like ?")
    .bind(title)
    .fetch_all(&*conn)
    .await;
    println!("{}", rows.is_ok());
    rows

}