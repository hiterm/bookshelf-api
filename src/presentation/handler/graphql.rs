use async_graphql::{
    dataloader::DataLoader,
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{Html, IntoResponse},
    Extension,
};

use crate::{
    extractors::Claims,
    presentation::graphql::{loader::AuthorLoader, mutation::Mutation, query::Query},
    use_case::traits::{mutation::MutationUseCase, query::QueryUseCase},
};

async fn graphql_handler<QUC, MUC>(
    schema: Extension<Schema<Query<QUC>, Mutation<MUC>, EmptySubscription>>,
    Extension(query_use_case): Extension<QUC>,
    claims: Claims,
    req: GraphQLRequest,
) -> GraphQLResponse
where
    QUC: QueryUseCase + Clone,
    MUC: MutationUseCase,
{
    let query_use_case: QUC = query_use_case.clone();
    let author_loader = DataLoader::new(
        AuthorLoader::new(claims.clone(), query_use_case),
        actix_web::rt::spawn,
    );

    schema
        .execute(req.into_inner().data(claims).data(author_loader))
        .await
        .into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}
