use actix_web::web;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::{
    extractors::Claims,
    presentational::graphql::{query::Query, query_service::QueryService},
};

pub async fn graphql<QS>(
    schema: web::Data<Schema<Query<QS>, EmptyMutation, EmptySubscription>>,
    request: GraphQLRequest,
    claims: Claims,
) -> GraphQLResponse
where
    QS: QueryService,
{
    schema
        .execute(request.into_inner().data(claims))
        .await
        .into()
}
