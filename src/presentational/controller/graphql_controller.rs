use actix_web::{get, web, HttpResponse, Responder};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema,
};
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

#[get("/graphql/playground")]
pub async fn graphql_playground() -> impl Responder {
    let source = playground_source(GraphQLPlaygroundConfig::new("/graphql"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(source)
}
