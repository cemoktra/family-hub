//! This module handles what happens in the kitchen for the family.
//! - [x] parse recipes from URLs
//! - [ ] cooking book as a collection of recipes
//! - [ ] weekly food plan referencing recipes
//! - [ ] shopping list created from weekly food plan
mod cooking_book;
mod error;
mod recipe;

pub use cooking_book::CookingBook;
pub use error::Error;
pub use recipe::Recipe;
