#![allow(non_snake_case)]
use std::time::UNIX_EPOCH;

pub use crate::backend::*;
use regex::Regex;
use html_escape::decode_html_entities;

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
    pub duration: Option<u32>,  // in minutes
    pub popularity: Option<u64>,
    pub averageScore: Option<u32>,
    pub synonyms: Option<Vec<String>>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<Tag>>,
    pub studios: Option<Studios>,
    pub relations: Option<Vec<Relations>>,
    pub characters: Option<Vec<Characters>>,
    pub recommendations: Option<Vec<Recommendations>>,
    pub bannerImage: Option<String>,
    pub coverImage: Option<CoverImage>,
    pub nextAiringEpisode: Option<NextAiringEpisode>,
    pub updatedAt: Option<i64>
}

impl Anime {
    pub fn get_updated_at(&self) -> i64 {
        let current = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        self.updatedAt.unwrap_or(current)
    }

    pub fn get_title_romaji(&self) -> &str{
        self.title.romaji.as_deref().unwrap_or("UNKNOWN")
    }

    pub fn get_title_english(&self)-> &str{
        self.title.english.as_deref().unwrap_or("UNKNOWN")
    }

    pub fn get_description(&self)-> String{
        let description = self.description.as_deref().unwrap_or("UNKNOWN");
        if description.trim().is_empty() {
            return "UNKNOWN".to_string()
        }

        // Regex to strip all HTML tags like <i>, <br>, etc.
        let re = Regex::new(r"<[^>]*>").unwrap();
        let no_tags = re.replace_all(description, "");

        // Decode HTML entities like &amp;, &#39;, etc.
        let decoded = decode_html_entities(&no_tags);

        return decoded.trim().to_string();
    
    }
    pub fn get_format(&self)-> &str{
        self.format.as_deref().unwrap_or("UNKNOWN")
    }
    pub fn get_episodes(&self)-> i32{
        match self.episodes {
            Some(ep) => ep as i32,
            None => -1,
        }
    }
    pub fn get_status(&self)-> &str{
        self.status.as_deref().unwrap_or("UNKNOWN")
    }
    pub fn get_start_date(&self)-> String{
        if let Some(date) = &self.startDate {
            if let (Some(day), Some(month), Some(year)) = (date.day, date.month, date.year) {
                return format!("{}-{}-{}", day, month, year);
            }
        }
        "UNKNOWN".to_string()
    }
    pub fn get_end_date(&self)-> String{
        if let Some(date) = &self.startDate {
            if let (Some(day), Some(month), Some(year)) = (date.day, date.month, date.year) {
                return format!("{}-{}-{}", day, month, year);
            }
        }
        "UNKNOWN".to_string()
    }

    pub fn get_season(&self)-> &str{
        self.season.as_deref().unwrap_or("UNKNOWN")
    }
    
    pub fn get_season_year(&self)-> i32{
        match self.seasonYear {
            Some(year) => year as i32,
            None => -1 
        }
    }
    
    pub fn get_duration(&self)-> i32{
        match self.duration {
            Some(duration) => duration as i32,
            None => -1
        }
    }
    
    pub fn get_popularity(&self)-> i64{
        match self.popularity {
            Some(popularity) => popularity as i64,
            None => -1
        }
    }
    
    pub fn get_averageScore(&self)-> i32{
        match self.averageScore {
            Some(score) => score as i32,
            None => -1
        }
    }

    pub fn get_synonyms(&self)-> &[String]{
        self.synonyms.as_deref().unwrap_or(&[])
    }

    pub fn get_genres(&self)-> &[String]{
        self.genres.as_deref().unwrap_or(&[])
    }

    pub fn get_tags(&self) -> &[Tag] {
        self.tags.as_deref().unwrap_or(&[])
    }

    pub fn get_studios(&self) -> Vec<String> {
        if let Some(stud) = &self.studios{
            let studNode: Vec<String> = stud.nodes.iter()
            .map(|node| node.name.clone().unwrap_or("UNKNOWN".to_string())).collect();
            return studNode;
        }
        vec![]
    }

    pub fn get_recommended(&self)-> Vec<(String, i32)>{
        let mut recommendations = Vec::new();

        if let Some(recommendations_data) = &self.recommendations {
            for data in recommendations_data {
                let node = &data.nodes;
                for node_data in node{
                    let title = node_data
                        .media
                        .as_ref()
                        .and_then(|m| m.title.romaji.clone())
                        .unwrap_or_else(|| "UNKNOWN".to_string());

                    let rating = node_data.rating.unwrap_or(-1);

                    recommendations.push((title, rating));
                }
            }   
        }

        recommendations
    }

    pub fn get_related(&self)->Vec<(&str, &str)> {
        let mut relations = vec![];

        if let Some(rel) = &self.relations{
            for relation in rel {
                let relation_edges = &relation.edges;
                for edges in relation_edges {
                    let realtion_type = edges.relationType.as_deref().unwrap_or("UNKNOWN");
                    let relation_name = edges.node.as_ref()
                    .and_then(|n| n.title.romaji.as_deref()).unwrap_or("UNKNOWN");
                    relations.push((relation_name, realtion_type));
                }
            }
        }
        relations
    }

    pub fn get_characters(&self) -> Vec<(&str,&str , &str)>{ //name, role, image
        let mut characters = vec![];

        if let Some(character_vec) = &self.characters{
            for character in character_vec{
                for edge in &character.edges{
                    let role = edge.role.as_deref().unwrap_or("UNKNOWN");
                    let name = edge.node.name.full.as_deref().unwrap_or("UNKNOWN");
                    let image = edge.node.image.as_ref().and_then(|img| img.medium.as_deref()).unwrap_or("UNKNWON");
                    characters.push((name, role, image));
                }
            }
        }
        characters
    }

    pub fn get_bannner_image(&self) -> &str {
        self.bannerImage.as_deref().unwrap_or("UNKNOWN")
    }

    pub fn get_airing_at(&self) -> (i32, i64) {
        if let Some(ref next) = self.nextAiringEpisode {
            (
                next.episode.unwrap_or(-1),
                next.airingAt.unwrap_or(-1)
            )
        } else {
            (-1, -1)
        }
    }

    pub fn get_cover_images(&self) -> (&str, &str, &str) {
        if let Some(ref cover) = self.coverImage {
            (
                cover.ExtraLargeImage.as_deref().unwrap_or("UNKNOWN"),
                cover.LargeImage.as_deref().unwrap_or("UNKNOWN"),
                cover.mediumImage.as_deref().unwrap_or("UNKNOWN"),
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
#[derive(Deserialize, Serialize, Default )]
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

#[derive(Deserialize, Serialize, Default)]
pub struct Studios {
    pub nodes: Vec<StudioNode>,
}

#[derive(Deserialize, Serialize )]
pub struct StudioNode {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Tag{
    pub name: Option<String>,
    pub rank: Option<i32>,
    pub isAdult: Option<bool>
}

#[derive(Deserialize, Serialize,Default)]
pub struct CoverImage {
    pub mediumImage: Option<String>,
    pub LargeImage: Option<String>,
    pub ExtraLargeImage: Option<String>,
}

#[derive(Deserialize, Serialize )]
pub struct Relations {
    pub edges: Vec<RelationEdge>,
}

#[derive(Deserialize, Serialize )]
pub struct RelationEdge {
    pub relationType: Option<String>,
    pub node: Option<RelatedAnime>,
}

#[derive(Deserialize, Serialize, Default )]
pub struct RelatedAnime {
    pub title: Title,
}

#[derive(Deserialize, Serialize )]
pub struct Characters {
    pub edges: Vec<CharacterEdge>,
}

#[derive(Deserialize, Serialize )]
pub struct CharacterEdge {
    pub role: Option<String>,
    pub node: CharacterNode,
}

#[derive(Deserialize, Serialize )]
pub struct CharacterNode {
    pub name: CharacterName,
    pub image: Option<CharacterImage>,
}

#[derive(Deserialize, Serialize )]
pub struct CharacterName {
    pub full: Option<String>,
}

#[derive(Deserialize, Serialize, Default )]
pub struct CharacterImage {
    pub medium: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Recommendations {
    pub nodes: Vec<RecommendationNode>,
}

#[derive(Deserialize, Serialize )]
pub struct RecommendationNode {
    pub rating: Option<i32>,
    pub media: Option<RecommendationMedia>,
}

#[derive(Deserialize, Serialize )]
pub struct RecommendationMedia {
    pub title: Title,
}
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct  NextAiringEpisode{
    pub episode: Option<i32>,
    pub airingAt: Option<i64>,
}