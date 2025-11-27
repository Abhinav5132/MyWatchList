
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
    extraLargeImage TEXT,
    extraLargeImageLocal BLOB,
    LargeImage TEXT,
    LargeImageLocal BLOB,
    mediumImage TEXT,
    mediumImageLocal BLOB,
    banner_image TEXT,
    duration INT,
    popularity INT,
    averageScore FLOAT,
    next_episode TEXT,
    next_episode_airing_at TEXT,
    updatedAt INTEGER
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
    related_name TEXT NOT NULL,
    relation_type TEXT,
    Foreign Key (anime_id) REFERENCES anime(id)

);

CREATE Table IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tag Text,
    rank INTEGER,
    isAdult INTEGER
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
    Foreign Key (genre_id) REFERENCES genres(id)
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
    id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_name TEXT NOT NULL UNIQUE,
    user_email TEXT NOT NULL UNIQUE,
    user_password TEXT NOT NULL,
    user_access_token TEXT,
    user_refresh_token TEXT,
    user_pfp TEXT NOT NULL -- should be not null in production
);

CREATE TABLE IF NOT EXISTS friends (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_1 INT NOT NULL,
    user_2 INT NOT NULL,
    CHECK (user_1 < user_2),
    FOREIGN KEY (user_1) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (user_2) REFERENCES user(id) ON DELETE CASCADE,
    UNIQUE(user_1, user_2)
);

CREATE TABLE IF NOT EXISTS friend_requests (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- request id
    status TEXT NOT NULL, 
    sender_id INT NOT NULL,      
    receiver_id INT NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (sender_id) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (receiver_id) REFERENCES user(id) ON DELETE CASCADE,
    UNIQUE(sender_id, receiver_id)
);

Create Table IF NOT EXISTS watch_list( --unordered watch-list 
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    user_id INTEGER NOT NUll,
    privacy_type TEXT NOT Null,
    is_ranked INT NOT NULL, -- 0 for not ranked and 1 for ranked.
    list_image TEXT NOT NULL, --text instead of blob to directly store base 64 encoded image
    is_user_image INT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    UNIQUE(user_id, name) -- no duplicate list names per user
);

CREATE TABLE IF NOT EXISTS watch_list_anime (
    user_id INTEGER NOT NULL,
    list_id INTEGER NOT NULL,
    anime_id INTEGER NOT NULL,
    rank INTEGER, -- add date added to allow sorting through date
    PRIMARY KEY (user_id, list_id, anime_id),
    FOREIGN KEY (list_id) REFERENCES watch_list(id) ON DELETE CASCADE,
    FOREIGN KEY (anime_id) REFERENCES anime(id),
    UNIQUE(user_id, list_id, rank)
);