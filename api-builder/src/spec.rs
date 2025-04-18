mod kitchen;

use indexmap::IndexMap;
use kitchen::Kitchen;
use openapiv3::{Info, OpenAPI};

#[derive(Debug)]
pub(crate) struct Specification {
    openapi: OpenAPI,
}

impl Specification {
    pub fn new() -> Self {
        let version = env!("CARGO_PKG_VERSION");

        let mut slf = Self {
            openapi: OpenAPI {
                openapi: "3.0.0".into(),
                info: Info {
                    title: "Family Hub".into(),
                    version: version.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        };

        slf.security();

        Kitchen::specify(
            slf.openapi.components.get_or_insert(Default::default()),
            &mut slf.openapi.paths,
        );

        slf
    }

    pub fn json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(&self.openapi)
    }

    fn security(&mut self) {
        let key: String = "family_hub_security".into();

        let mut security = IndexMap::new();
        security.insert(key.clone(), vec![]);
        self.openapi.security = Some(vec![security]);

        self.openapi
            .components
            .get_or_insert(Default::default())
            .security_schemes
            .insert(
                key,
                openapiv3::ReferenceOr::Item(openapiv3::SecurityScheme::HTTP {
                    scheme: "bearer".into(),
                    bearer_format: Some("JWT".into()),
                    description: None,
                    extensions: std::default::Default::default(),
                }),
            );
    }
}
