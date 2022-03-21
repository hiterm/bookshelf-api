use actix_web::{post, web};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::{
    extractors::Claims,
    presentational::graphql::{query::QueryRoot, query_service::QueryServiceImpl},
};

#[post("/graphql")]
pub async fn graphql(
    schema: web::Data<Schema<QueryRoot<QueryServiceImpl>, EmptyMutation, EmptySubscription>>,
    request: GraphQLRequest,
    _claims: Claims,
) -> GraphQLResponse {
    schema.execute(request.into_inner()).await.into()
}
