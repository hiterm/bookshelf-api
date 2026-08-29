use bookshelf_api::{
    presentation::graphql::{mutation::Mutation, query::Query, schema::build_schema},
    use_case::traits::{
        author::{MockAuthorCommandUseCase, MockAuthorQueryUseCase},
        book::{MockBookCommandUseCase, MockBookQueryUseCase},
        history::{MockHistoryCommandUseCase, MockHistoryQueryUseCase},
        user::{MockUserCommandUseCase, MockUserQueryUseCase},
    },
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let query = Query::new(
        MockUserQueryUseCase::new(),
        MockBookQueryUseCase::new(),
        MockAuthorQueryUseCase::new(),
        MockHistoryQueryUseCase::new(),
    );
    let mutation = Mutation::new(
        MockUserCommandUseCase::new(),
        MockBookCommandUseCase::new(),
        MockAuthorCommandUseCase::new(),
        MockHistoryCommandUseCase::new(),
    );
    let schema = build_schema(query, mutation);
    let sdl = format!("{}\n", schema.sdl());
    std::fs::write("schema.graphql", &sdl).expect("write generated GraphQL schema");
    print!("{sdl}");
}
