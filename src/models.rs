use serde_derive::{Deserialize, Serialize};

pub trait HasId {
    fn get_id(&self) -> String;
}

pub trait HasName {
    fn get_name() -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub parent_id: String,
    pub season: i32,
    pub episode: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Title {
    #[serde(alias = "tconst")]
    pub id: String,
    #[serde(alias = "titleType")]
    pub title_type: String,
    #[serde(alias = "primaryTitle")]
    pub primary_title: String,
    #[serde(alias = "originalTitle")]
    pub original_title: String,
    #[serde(alias = "startYear")]
    pub start_year: String,
    #[serde(alias = "endYear")]
    #[serde(skip_serializing_if = "is_invalid")]
    pub end_year: Option<String>,
    #[serde(alias = "runtimeMinutes")]
    #[serde(skip_serializing_if = "is_invalid")]
    pub runtime_minutes: Option<String>,
    #[serde(skip_serializing_if = "is_invalid")]
    pub genres: Option<String>,
}
/*
#[derive(Debug, Serialize, Deserialize)]
pub struct Title {
    #[serde(alias = "tconst")]
    pub id: String,
    #[serde(alias = "titleType")]
    pub title_type: String,
    #[serde(alias = "primaryTitle")]
    pub primary_title: String,
    #[serde(alias = "originalTitle")]
    pub original_title: String,
    #[serde(alias = "startYear")]
    pub start_year: String,
    #[serde(alias = "endYear")]
    pub end_year: Option<String>,
    #[serde(alias = "runtimeMinutes")]
    pub runtime_minutes: Option<String>,
    pub genres: Option<String>,
}
*/
impl HasId for Title {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl HasName for Title {
    fn get_name() -> &'static str {
        "title.basics.tsv.gz"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rating {
    pub id: String,
    pub avarage_rating: f32,
    pub num_votes: i32,
}

fn is_invalid(value: &Option<String>) -> bool {
    match value {
        Some(inner_value) => inner_value == "\\N",
        None => false,
    }
}
