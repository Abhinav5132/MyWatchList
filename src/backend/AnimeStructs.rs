#![allow(non_snake_case)]
use std::time::{SystemTime, UNIX_EPOCH};

pub use crate::backend::*;
use html_escape::decode_html_entities;
use regex::Regex;

#[derive(Deserialize, Serialize, Default)]
pub struct Studios {
    pub nodes: Vec<StudioNode>,
}

#[derive(Deserialize, Serialize)]
pub struct StudioNode {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Tag {
    pub name: Option<String>,
    pub rank: Option<i32>,
    pub isAdult: Option<bool>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct CoverImage {
    pub medium: Option<String>,
    pub large: Option<String>,
    pub extraLarge: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Relations {
    pub edges: Vec<RelationEdge>,
}

#[derive(Deserialize, Serialize)]
pub struct RelationEdge {
    pub relationType: Option<String>,
    pub node: Option<RelatedAnime>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct RelatedAnime {
    pub title: Title,
}

#[derive(Deserialize, Serialize)]
pub struct Characters {
    pub edges: Vec<CharacterEdge>,
}

#[derive(Deserialize, Serialize)]
pub struct CharacterEdge {
    pub role: Option<String>,
    pub node: CharacterNode,
}

#[derive(Deserialize, Serialize)]
pub struct CharacterNode {
    pub name: CharacterName,
    pub image: Option<CharacterImage>,
}

#[derive(Deserialize, Serialize)]
pub struct CharacterName {
    pub full: Option<String>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct CharacterImage {
    pub medium: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Recommendations {
    pub nodes: Vec<RecommendationNode>,
}

#[derive(Deserialize, Serialize)]
pub struct RecommendationNode {
    pub mediaRecommendation: Option<RecommendationMedia>,
}

#[derive(Deserialize, Serialize)]
pub struct RecommendationMedia {
    pub title: Title,
}
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct NextAiringEpisode {
    pub episode: Option<i32>,
    pub airingAt: Option<i64>,
}

pub trait PartialUpdate {
    fn updated_at(&self) -> Option<i64>;
    fn episodes(&self) -> Option<u32>;
    fn status(&self) -> Option<&str>;
    fn end_date(&self) -> Option<&Date>;
    fn popularity(&self) -> Option<u64>;
    fn average_score(&self) -> Option<u32>;
    fn relations(&self) -> Option<&Relations>;
    fn next_airing(&self) -> Option<&NextAiringEpisode>;
    fn recommendations(&self) -> Option<&Recommendations>;

    fn get_updated_at(&self) -> i64 {
        self.updated_at().unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        })
    }

    fn get_episodes(&self) -> i32 {
        self.episodes().map(|e| e as i32).unwrap_or(-1)
    }

    fn get_status(&self) -> &str {
        self.status().unwrap_or("UNKNOWN")
    }

    fn get_popularity(&self) -> i64 {
        self.popularity().map(|p| p as i64).unwrap_or(-1)
    }

    fn get_average_score(&self) -> i32 {
        self.average_score().map(|s| s as i32).unwrap_or(-1)
    }

    fn get_airing_at(&self) -> (i32, i64) {
        if let Some(next) = self.next_airing() {
            (
                next.episode.unwrap_or(-1),
                next.airingAt.unwrap_or(-1),
            )
        } else {
            (-1, -1)
        }
    }

    fn get_related(&self) -> Vec<(&str, &str)> {
        let mut out = Vec::new();

        if let Some(rel) = self.relations() {
            for edge in &rel.edges {
                let relation_type = edge.relationType.as_deref().unwrap_or("UNKNOWN");
                let title = edge
                    .node
                    .as_ref()
                    .and_then(|n| n.title.romaji.as_deref())
                    .unwrap_or("UNKNOWN");
                out.push((title, relation_type));
            }
        }

        out
    }

    fn get_recommended(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(recommendations_data) = &self.recommendations() {
            let node = &recommendations_data.nodes;
            for node_data in node {
                let title = node_data
                    .mediaRecommendation
                    .as_ref()
                    .and_then(|m| m.title.romaji.clone())
                    .unwrap_or_else(|| "UNKNOWN".to_string());

                recommendations.push(title);
            }
        }

        recommendations
    }

    fn get_end_date(&self) -> String {
        if let Some(date) = self.end_date()
            && let (Some(d), Some(m), Some(y)) = (date.day, date.month, date.year)
        {
            format!("{d}-{m}-{y}")
        } else {
            "UNKNOWN".to_string()
        }
    }
}



#[derive(Deserialize, Serialize)]
pub struct Anime {
    pub title: Title,
    pub description: Option<String>,
    pub format: Option<String>,
    pub episodes: Option<u32>,
    pub status: Option<String>,
    pub startDate: Option<Date>,
    pub endDate: Option<Date>,
    pub season: Option<String>,
    pub seasonYear: Option<u32>,
    pub duration: Option<u32>, // in minutes
    pub popularity: Option<u64>,
    pub averageScore: Option<u32>,
    pub synonyms: Option<Vec<String>>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<Tag>>,
    pub studios: Option<Studios>,
    pub relations: Option<Relations>,
    pub characters: Option<Characters>,
    pub recommendations: Option<Recommendations>,
    pub bannerImage: Option<String>,
    pub coverImage: Option<CoverImage>,
    pub nextAiringEpisode: Option<NextAiringEpisode>,
    pub updatedAt: Option<i64>,
}

impl PartialUpdate for Anime {

    fn recommendations(&self) -> Option<&Recommendations> {
        self.recommendations.as_ref()
    }
    fn updated_at(&self) -> Option<i64> {
        self.updatedAt
    }

    fn episodes(&self) -> Option<u32> {
        self.episodes
    }

    fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    fn end_date(&self) -> Option<&Date> {
        self.endDate.as_ref()
    }

    fn popularity(&self) -> Option<u64> {
        self.popularity
    }

    fn average_score(&self) -> Option<u32> {
        self.averageScore
    }

    fn relations(&self) -> Option<&Relations> {
        self.relations.as_ref()
    }

    fn next_airing(&self) -> Option<&NextAiringEpisode> {
        self.nextAiringEpisode.as_ref()
    }
}


impl Anime {

    pub fn get_title_romaji(&self) -> &str {
        self.title.romaji.as_deref().unwrap_or("UNKNOWN")
    }

    pub fn get_title_english(&self) -> &str {
        self.title.english.as_deref().unwrap_or("UNKNOWN")
    }

    pub fn get_description(&self) -> String {
        let description = self.description.as_deref().unwrap_or("UNKNOWN");
        if description.trim().is_empty() {
            return "UNKNOWN".to_string();
        }

        // Regex to strip all HTML tags like <i>, <br>, etc.
        let re = Regex::new(r"<[^>]*>").unwrap();
        let no_tags = re.replace_all(description, "");

        // Decode HTML entities like &amp;, &#39;, etc.
        let decoded = decode_html_entities(&no_tags);

        decoded.trim().to_string()
    }
    pub fn get_format(&self) -> &str {
        self.format.as_deref().unwrap_or("UNKNOWN")
    }
   
    pub fn get_start_date(&self) -> String {
        if let Some(date) = &self.startDate
            && let (Some(day), Some(month), Some(year)) = (date.day, date.month, date.year) {
                return format!("{}-{}-{}", day, month, year);
            }
        "UNKNOWN".to_string()
    }

    pub fn get_season(&self) -> &str {
        self.season.as_deref().unwrap_or("UNKNOWN")
    }

    pub fn get_season_year(&self) -> i32 {
        match self.seasonYear {
            Some(year) => year as i32,
            None => -1,
        }
    }

    pub fn get_duration(&self) -> i32 {
        match self.duration {
            Some(duration) => duration as i32,
            None => -1,
        }
    }

    pub fn get_synonyms(&self) -> &[String] {
        self.synonyms.as_deref().unwrap_or(&[])
    }

    pub fn get_genres(&self) -> &[String] {
        self.genres.as_deref().unwrap_or(&[])
    }

    pub fn get_tags(&self) -> &[Tag] {
        self.tags.as_deref().unwrap_or(&[])
    }

    pub fn get_studios(&self) -> Vec<String> {
        if let Some(stud) = &self.studios {
            let studNode: Vec<String> = stud
                .nodes
                .iter()
                .map(|node| node.name.clone().unwrap_or("UNKNOWN".to_string()))
                .collect();
            return studNode;
        }
        vec![]
    }

    pub fn get_characters(&self) -> Vec<(&str, &str, &str)> {
        //name, role, image
        let mut characters = vec![];

        if let Some(character_vec) = &self.characters {
            for edge in &character_vec.edges {
                let role = edge.role.as_deref().unwrap_or("UNKNOWN");
                let name = edge.node.name.full.as_deref().unwrap_or("UNKNOWN");
                let image = edge
                    .node
                    .image
                    .as_ref()
                    .and_then(|img| img.medium.as_deref())
                    .unwrap_or("UNKNOWN");
                characters.push((name, role, image));
            }
        }
        characters
    }

    pub fn get_bannner_image(&self) -> &str {
        self.bannerImage.as_deref().unwrap_or("UNKNOWN")
    }

    pub fn get_cover_images(&self) -> (&str, &str, &str) {
        if let Some(ref cover) = self.coverImage {
            (
                cover.extraLarge.as_deref().unwrap_or("UNKNOWN"),
                cover.large.as_deref().unwrap_or("UNKNOWN"),
                cover.medium.as_deref().unwrap_or("UNKNOWN"),
            )
        } else {
            ("UNKNOWN", "UNKNOWN", "UNKNOWN")
        }
    }
}

//Nested Types
#[derive(Deserialize, Serialize, Default)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}
#[derive(Deserialize, Serialize, Default)]
pub struct Date {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

impl Date {
    pub fn construct_date(self) -> String {
        todo!()
    }
}

