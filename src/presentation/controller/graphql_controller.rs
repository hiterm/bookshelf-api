use actix_web::{get, web, HttpResponse, Responder};
use async_graphql::{
    dataloader::DataLoader,
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::{
    presentation::{
        extractor::claims::Claims,
        graphql::{loader::AuthorLoader, mutation::Mutation, query::Query},
    },
    use_case::traits::{mutation::MutationUseCase, query::QueryUseCase},
};

pub async fn graphql<QUC, MUC>(
    schema: web::Data<Schema<Query<QUC>, Mutation<MUC>, EmptySubscription>>,
    query_use_case: web::Data<QUC>,
    request: GraphQLRequest,
    claims: Claims,
) -> GraphQLResponse
where
    QUC: QueryUseCase + Clone,
    MUC: MutationUseCase,
{
    let query_use_case: QUC = query_use_case.as_ref().clone();
    let author_loader = DataLoader::new(
        AuthorLoader::new(claims.clone(), query_use_case),
        actix_web::rt::spawn,
    );

    schema
        .execute(request.into_inner().data(claims).data(author_loader))
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
