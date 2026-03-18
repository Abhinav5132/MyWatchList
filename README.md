# MyWatchList

An anime watchlist application built completely in Rust, featuring a local backend and a native desktop interface.

<img width="1916" height="1039" alt="App Screenshot" src="https://github.com/user-attachments/assets/b49f2856-a3ae-47e7-b05a-bd28f9efa011" />

*(You can add more screenshots here later)*

## Features

- **Watch Tracking**: Keep track of the anime you are watching, have completed, or plan to watch.
- **Detailed Anime Data**: Uses data sourced from AniList.
- **Custom Lists**: Create and manage customizable watchlists.
- **Friends System**: Add friends, view their profiles, and see their watchlists.
- **Local Database**: Built with SQLite for fast, local data access and responsiveness.
- **Cross-Platform**: Designed primarily for desktop via the Dioxus framework, with core logic separated for future web/mobile support.

## Getting Started

### Prerequisites

- [Rust Toolchain](https://rustup.rs/) (Edition 2024 is specified in Cargo.toml)
- OpenSSL

### Installation & Execution

1. **Clone the repository**
   ```bash
   git clone https://github.com/Abhinav5132/MyWatchList.git
   cd MyWatchList
   ```

2. **Run the application**
   Because MyWatchList bundles both the frontend desktop client and the backend server together, you can launch the app with a single standard cargo command:
   ```bash
   cargo run
   ```
   *This command initializes the local backend on a background thread and launches the Dioxus frontend.*

## Project Structure

- `src/frontend/` - Dioxus pages, components, routing, and UI logic.
- `src/backend/` - Actix routes, database queries, authentication handling, and structural models.
- `anime.db` / `anime.sql` - Local SQLite databases handling app state and cached anime data.

## Roadmap

There are several planned improvements for MyWatchList, including:
- Improving image load times and caching.
- Direct file linking to track watch progress from local video files.
- UI enhancements (tag percentages, search result feedback, and styling fixes).

For a complete and up-to-date look at what's being actively worked on, check the `TODO.txt` file in the root directory.
