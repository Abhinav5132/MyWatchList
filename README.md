# MyWatchList

An anime watchlist application built completely in Rust, featuring a local backend and a native desktop interface.
Home page: 
<img width="1918" height="1049" alt="image" src="https://github.com/user-attachments/assets/3f505d3c-13f0-4437-bf29-398a91ef90c1" />

Details page(UI under development):
<img width="1917" height="1047" alt="image" src="https://github.com/user-attachments/assets/3e3d22e7-f110-4d2e-94bc-4bf7703d3acf" />


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
   dx serve
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
