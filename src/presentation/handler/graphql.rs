use async_graphql::{
    dataloader::DataLoader,
    http::{GraphQLPlaygroundConfig, playground_source},
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Extension,
    response::{Html, IntoResponse},
};

use crate::{
    dependency_injection::{AQ, AppSchema, BQ},
    presentation::{
        extractor::claims::Claims,
        graphql::loader::{AuthorLoader, BooksByAuthorLoader},
    },
};

pub async fn graphql_handler(
    claims: Claims,
    schema: Extension<AppSchema>,
    Extension(author_query): Extension<AQ>,
    Extension(book_query): Extension<BQ>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let author_loader = DataLoader::new(
        AuthorLoader::new(claims.clone(), author_query),
        tokio::spawn,
    );
    let books_by_author_loader = DataLoader::new(
        BooksByAuthorLoader::new(claims.clone(), book_query),
        tokio::spawn,
    );

    schema
        .execute(
            req.into_inner()
                .data(claims)
                .data(author_loader)
                .data(books_by_author_loader),
        )
        .await
        .into()
}

pub async fn graphql_playground_handler() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}
