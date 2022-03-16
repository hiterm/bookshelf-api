use actix_web::{post, web};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::{extractors::Claims, presentational::graphql::schema::Query};

#[post("/graphql")]
pub async fn graphql(
    schema: web::Data<Schema<Query, EmptyMutation, EmptySubscription>>,
    request: GraphQLRequest,
    _claims: Claims,
) -> GraphQLResponse {
    schema.execute(request.into_inner()).await.into()
}
