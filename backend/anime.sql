
CREATE TABLE IF NOT EXISTS anime (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    format TEXT,
    episodes INT,
    status TEXT,
    anime_season TEXT,
    anime_year INT,
    picture TEXT,
    thumbnail TEXT,
    duration INT,
    score FLOAT
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

Create Table IF NOT EXISTS producers(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE
);

CREATE Table IF NOT EXISTS anime_producers(
    anime_id INT,
    producer_id INT,
    Foreign Key (anime_id) REFERENCES anime(id),
    Foreign Key (producer_id) REFERENCES producers(id)
);

CREATE TABLE IF NOT EXISTS anime_character (
    anime_id INTEGER NOT NULL,
    character_id INTEGER NOT NULL,
    role TEXT,
    FOREIGN KEY (anime_id) REFERENCES anime(id),
    FOREIGN KEY (character_id) REFERENCES characters(id),
    PRIMARY KEY (anime_id, character_id)
);

CREATE TABLE IF NOT EXISTS character_voice_actor (
    character_id INTEGER NOT NULL,
    voice_actor_id INTEGER NOT NULL,
    anime_id INTEGER NOT NULL,
    FOREIGN KEY (character_id) REFERENCES characters(id),
    FOREIGN KEY (voice_actor_id) REFERENCES voice_actor(id),
    FOREIGN KEY (anime_id) REFERENCES anime(id),
    PRIMARY KEY (character_id, voice_actor_id, anime_id)
);

CREATE TABLE IF NOT EXISTS characters (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    image_url TEXT
);

CREATE TABLE IF NOT EXISTS voice_actor (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    language TEXT,
    image_url TEXT
);