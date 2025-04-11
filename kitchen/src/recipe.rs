//! This module handles downloading/parsing of google recipe schema:
//! https://developers.google.com/search/docs/appearance/structured-data/recipe?hl=de
//!
use std::string::FromUtf8Error;

use headers::Header;
use iso8601::Duration;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> From<OneOrMany<T>> for Vec<T> {
    fn from(value: OneOrMany<T>) -> Self {
        match value {
            OneOrMany::One(value) => vec![value],
            OneOrMany::Many(values) => values,
        }
    }
}

impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        Self::Many(vec![])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Instructions {
    One(String),
    Many(Vec<String>),
    HowToSteps(Vec<HowToStep>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HowToStep {
    pub text: String,
}

impl From<Instructions> for Vec<String> {
    fn from(value: Instructions) -> Self {
        match value {
            Instructions::One(item) => vec![item],
            Instructions::Many(items) => items,
            Instructions::HowToSteps(how_to_steps) => {
                how_to_steps.into_iter().map(|item| item.text).collect()
            }
        }
    }
}

impl Default for Instructions {
    fn default() -> Self {
        Self::Many(vec![])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub image: OneOrMany<Url>,
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: OneOrMany<String>,
    #[serde(rename = "recipeCuisine", default)]
    pub recipe_cuisine: OneOrMany<String>,
    #[serde(rename = "recipeCategory", default)]
    pub recipe_category: OneOrMany<String>,
    #[serde(rename = "recipeIngredient", default)]
    pub recipe_ingredient: Vec<String>,
    #[serde(rename = "recipeInstructions", default)]
    pub recipe_instructions: Instructions,
    #[serde(rename = "cookTime")]
    pub cook_time: Option<Duration>,
    #[serde(rename = "prepTime")]
    pub prep_time: Option<Duration>,
    #[serde(rename = "totalTime")]
    pub total_time: Option<Duration>,
}

#[derive(Debug, thiserror::Error)]
pub enum RecipeError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
    #[error(transparent)]
    Scraper(Box<dyn core::error::Error>),
    #[error("No recipe parsed")]
    NoRecipe,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl Recipe {
    pub async fn from_url(url: Url) -> Result<Self, RecipeError> {
        let client = reqwest::Client::new();

        tracing::info!("receiving data from {url} ...");

        let result = client
            .get(url)
            .header(headers::UserAgent::name(), "Mozilla")
            .send()
            .await?;

        let status_code = result.status();
        let bytes = result.bytes().await?;
        tracing::info!("status = {status_code} - received {} bytes", bytes.len());

        let data = String::from_utf8(bytes.to_vec())?;
        let decoded = html_escape::decode_html_entities(&data);

        let document = scraper::Html::parse_document(&decoded);

        let selector = scraper::Selector::parse("script[type=\"application/ld+json\"]")
            .map_err(|err| RecipeError::Scraper(Box::new(err)))?;

        for element in document.select(&selector) {
            let inner = element.inner_html();

            let deser = &mut serde_json::Deserializer::from_str(&inner);
            match serde_path_to_error::deserialize(deser) {
                //match serde_json::from_value(json) {
                Ok(recipe) => return Ok(recipe),
                Err(err) => {
                    tracing::warn!("failed to parse json: {err} [{}]", err.path().to_string())
                }
            }
        }

        Err(RecipeError::NoRecipe)
    }
}
