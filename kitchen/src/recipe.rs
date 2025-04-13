//! This module handles downloading/parsing of google recipe schema:
//! https://developers.google.com/search/docs/appearance/structured-data/recipe?hl=de

use std::str::FromStr;

use headers::Header;
use iso8601::Duration;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::Error;

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

impl Recipe {
    /// parse a recipe from an existing URL, tested to work with:
    /// - chefkoch.de
    /// - cookidoo.de
    /// - kitchenstories.com
    ///
    /// but in general it should work with all websites containing the google recipe schema
    pub async fn from_url(url: Url) -> Result<Option<Self>, Error> {
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
            .map_err(|err| Error::Scraper(Box::new(err)))?;

        for element in document.select(&selector) {
            let inner = element.inner_html();

            match Self::from_str(&inner) {
                Ok(recipe) => return Ok(Some(recipe)),
                Err(err) => {
                    tracing::warn!("failed to parse json: {err} [{}]", err.path().to_string())
                }
            }
        }

        Ok(None)
    }
}

impl FromStr for Recipe {
    type Err = serde_path_to_error::Error<serde_json::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deser = &mut serde_json::Deserializer::from_str(s);
        Ok(serde_path_to_error::deserialize(deser)?)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::str::FromStr;

    pub(crate) fn test_recipe_str() -> String {
        serde_json::to_string(&test_recipe()).unwrap()
    }

    fn test_recipe() -> serde_json::Value {
        serde_json::json!(
            {
                "name": "Goulash",
                "image": "https://upload.wikimedia.org/wikipedia/commons/thumb/4/49/Gulasch.jpg/1024px-Gulasch.jpg",
                "keywords": "Goulash",
                "recipeCuisine": ["Europe", "Hungary"],
                "recipeIngredient": ["Beef", "Onion", "Paprika"],
                "totalTime": "P0DT2H40M"
            }
        )
    }

    #[test]
    fn test_from_str() {
        assert!(super::Recipe::from_str(&test_recipe_str()).is_ok());
    }
}
