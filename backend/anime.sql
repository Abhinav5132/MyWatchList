
CREATE TABLE IF NOT EXISTS anime (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title_english TEXT,
    title_romanji TEXT NOT NULL,
    description TEXT,
    format TEXT,
    episodes INT,
    status TEXT,
    start_date TEXT,
    end_date TEXT,
    anime_season TEXT,
    anime_year INT,
    thumbnail TEXT,
    picture TEXT,
    banner_image TEXT,
    duration INT,
    popularity INT,
    averageScore FLOAT,
    trailer_url TEXT,
    next_episode TEXT,
    next_episode_airing_at TEXT
);

Create Table If not Exists synonyms(
    id INTEGER PRIMARY Key AUTOINCREMENT,
    anime_id INTEGER NOT NULL,
    synonym TEXT NOT NULL,
    FOREIGN KEY(anime_id) REFERENCES anime(id)
);

CREATE TABLE IF NOT EXISTS related_anime (
    related_anime_id INTEGER PRIMARY KEY AUTOINCREMENT,
    anime_id INTEGER NOT NULL,
    related_name TEXT NOT NUll,
    relation_type TEXT,
    Foreign Key (anime_id) REFERENCES anime(id)

);

CREATE Table IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tag Text
);

Create Table If Not EXISTS anime_tags(
    anime_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    Foreign Key (anime_id) REFERENCES anime(id),
    Foreign Key (tag_id) REFERENCES tags(id)
);

CREATE Table IF NOT EXISTS genres (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    genre Text
);

Create Table If Not EXISTS anime_genre(
    anime_id INTEGER NOT NULL,
    genre_id INTEGER NOT NULL,
    Foreign Key (anime_id) REFERENCES anime(id),
    Foreign Key (genre_id) REFERENCES genre(id)
);
Create Table IF NOT EXISTS studios(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE
);

CREATE Table IF NOT EXISTS anime_studio(
    anime_id INT,
    studio_id INT,
    Foreign Key (anime_id) REFERENCES anime(id),
    Foreign Key (studio_id) REFERENCES studios(id)
);


-- Characters
CREATE TABLE IF NOT EXISTS characters (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL  
);


-- Anime ↔ Characters (role is like MAIN / SUPPORTING)
CREATE TABLE IF NOT EXISTS anime_character (
    anime_id INTEGER NOT NULL,
    character_id INTEGER NOT NULL,
    role TEXT,
    image TEXT,
    FOREIGN KEY(anime_id) REFERENCES anime(id),
    FOREIGN KEY(character_id) REFERENCES characters(id),
    PRIMARY KEY (anime_id, character_id)
);

-- Recommendations
CREATE TABLE IF NOT EXISTS recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anime_id INTEGER NOT NULL,
    recommended_title TEXT NOT NULL,
    rating INT,
    FOREIGN KEY(anime_id) REFERENCES anime(id)
);

Create TABLE IF NOT EXISTS user (
    id AUTO_INCREMENT NOT NULL,
    user_name TEXT NOT NULL,
    user_email TEXT NOT NULL,
    user_password TEXT NOT NULL,
    user_pfp TEXT NOT NULL
);

Create Table IF NOT Exists watch_list(
    watch_list_name TEXT NOT NULL,
    anime_id INT NOT NULL,
    user_id INT NOT NULL,
    PRIMARY KEY (user_id, anime_id),
    Foreign Key (user_id) REFERENCES user(id),
    Foreign Key (anime_id) REFERENCES anime(id)
);
