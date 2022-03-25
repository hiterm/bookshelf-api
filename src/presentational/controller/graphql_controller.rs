use actix_web::web;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::{
    extractors::Claims, presentational::graphql::query::Query,
    use_case::use_case::query::QueryUseCase,
};

pub async fn graphql<QUC>(
    schema: web::Data<Schema<Query<QUC>, EmptyMutation, EmptySubscription>>,
    request: GraphQLRequest,
    claims: Claims,
) -> GraphQLResponse
where
    QUC: QueryUseCase,
{
    schema
        .execute(request.into_inner().data(claims))
        .await
        .into()
}
