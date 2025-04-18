use indexmap::IndexMap;
use openapiv3::{
    Components, MediaType, ObjectType, Operation, PathItem, Paths, ReferenceOr, RequestBody,
    Response, Responses, Schema, SchemaData, SchemaKind, StringType, Type,
};

pub(crate) struct Recipes {}

const POST_RECIPE: &str = "post_recipe";
const RECIPES_PATH: &str = "/kitchen/recipes";
const RECIPES_REF: &str = "kitchen.recipes";

impl Recipes {
    pub(crate) fn specify(components: &mut Components, paths: &mut Paths) {
        let mut path = PathItem::default();
        path.post = Some(Self::post_recipe(components));

        paths
            .paths
            .insert(RECIPES_PATH.into(), ReferenceOr::Item(path));
    }

    pub(crate) fn post_recipe(components: &mut Components) -> Operation {
        let mut responses = Responses::default();

        responses.responses.insert(
            openapiv3::StatusCode::Code(201),
            ReferenceOr::Item(Response {
                description: "Successfully added recipe".into(),
                ..Default::default()
            }),
        );
        responses.responses.insert(
            openapiv3::StatusCode::Code(400),
            ReferenceOr::Item(Response {
                description: "Failed to extract recipe".into(),
                ..Default::default()
            }),
        );
        responses.responses.insert(
            openapiv3::StatusCode::Code(401),
            ReferenceOr::Item(Response {
                description: "Unauthorized".into(),
                ..Default::default()
            }),
        );

        let key: String = "family_hub_security".into();

        let mut security = IndexMap::new();
        security.insert(key.clone(), vec![]);

        let mut body_object = ObjectType {
            required: vec!["url".into()],
            ..Default::default()
        };

        body_object.properties.insert(
            key,
            ReferenceOr::Item(Box::new(Schema {
                schema_data: SchemaData {
                    nullable: false,
                    description: Some("URL to extract recipe from".into()),
                    ..Default::default()
                },
                schema_kind: SchemaKind::Type(Type::String(StringType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Unknown("uri".into()),
                    ..Default::default()
                })),
            })),
        );

        let mut body = RequestBody {
            description: Some("Request to extract recipe from URL".into()),
            required: true,
            ..Default::default()
        };
        body.content.insert(
            "application/json".into(),
            MediaType {
                schema: Some(ReferenceOr::Item(Schema {
                    schema_data: SchemaData {
                        nullable: false,
                        ..Default::default()
                    },
                    schema_kind: SchemaKind::Type(Type::Object(body_object)),
                })),
                ..Default::default()
            },
        );

        components
            .request_bodies
            .insert(format!("{RECIPES_REF}.response"), ReferenceOr::Item(body));

        let op = Operation {
            tags: vec!["kitchen".into()],
            summary: Some("Add recipe to collection".into()),
            operation_id: Some(POST_RECIPE.into()),
            request_body: Some(ReferenceOr::Reference {
                reference: format!("#/components/requestBodies/{RECIPES_REF}.response"),
            }),
            responses,
            security: Some(vec![security]),
            ..Default::default()
        };

        op
    }
}
