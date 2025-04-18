mod recipes;

use openapiv3::{Components, Paths};
use recipes::Recipes;

pub(crate) struct Kitchen {}

impl Kitchen {
    pub(crate) fn specify(components: &mut Components, paths: &mut Paths) {
        Recipes::specify(components, paths);
    }
}
