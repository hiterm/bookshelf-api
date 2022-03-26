use actix_web::web;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::{
    extractors::Claims,
    presentational::graphql::{mutation::Mutation, query::Query},
    use_case::use_case::{mutation::MutationUseCase, query::QueryUseCase},
};

pub async fn graphql<QUC, MUC>(
    schema: web::Data<Schema<Query<QUC>, Mutation<MUC>, EmptySubscription>>,
    request: GraphQLRequest,
    claims: Claims,
) -> GraphQLResponse
where
    QUC: QueryUseCase,
    MUC: MutationUseCase,
{
    schema
        .execute(request.into_inner().data(claims))
        .await
        .into()
}
