//! This module handles recipe collections and optionally syncs them to a file

use std::path::PathBuf;

use tokio::sync::{RwLock, RwLockReadGuard};

// a collection of recipes as simple as it gets
use crate::{Error, Recipe};

#[derive(Debug)]
pub struct CookingBook {
    // sync to that file
    storage: Option<PathBuf>,
    recipes: RwLock<Vec<Recipe>>,
}

impl CookingBook {
    pub fn memory() -> Self {
        Self {
            storage: None,
            recipes: RwLock::new(vec![]),
        }
    }

    pub fn file(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();

        tracing::info!("new cooking book with file storage at '{}'", path.display());
        let recipes = if std::path::Path::exists(&path) {
            tracing::info!("reading existing cooking book from '{}'", path.display());
            let content = std::fs::read(&path)?;
            serde_json::from_slice(&content)?
        } else {
            vec![]
        };

        Ok(Self {
            storage: Some(path),
            recipes: RwLock::new(recipes),
        })
    }

    pub async fn recipes(&self) -> RwLockReadGuard<'_, Vec<Recipe>> {
        self.recipes.read().await
    }

    pub async fn push(&self, recipe: Recipe) -> Result<(), Error> {
        let mut lock = self.recipes.write().await;
        lock.push(recipe);

        if let Some(storage) = &self.storage {
            let content = serde_json::to_vec(&*lock)?;
            std::fs::write(storage, content)?;
        }

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::str::FromStr;

    #[tokio::test]
    async fn test_in_memory() {
        let recipe = super::Recipe::from_str(&crate::recipe::test::test_recipe_str()).unwrap();

        let book = super::CookingBook::memory();
        book.push(recipe).await.unwrap();
        assert_eq!(1, book.recipes().await.len());
    }

    #[tokio::test]
    async fn test_file_storage() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("cooking_book.json");

        {
            let recipe = super::Recipe::from_str(&crate::recipe::test::test_recipe_str()).unwrap();

            let book = super::CookingBook::file(&path).unwrap();
            book.push(recipe).await.unwrap();
            assert_eq!(1, book.recipes().await.len());
        }

        {
            let book = super::CookingBook::file(&path).unwrap();
            assert_eq!(1, book.recipes().await.len());
        }
    }
}
