use actix_web::web;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::{
    extractors::Claims,
    presentational::graphql::{query::QueryRoot, query_service::QueryService},
};

pub async fn graphql<QS>(
    schema: web::Data<Schema<QueryRoot<QS>, EmptyMutation, EmptySubscription>>,
    request: GraphQLRequest,
    _claims: Claims,
) -> GraphQLResponse
where
    QS: QueryService,
{
    schema.execute(request.into_inner()).await.into()
}
