use bookshelf_api::{
    presentation::graphql::{mutation::Mutation, query::Query, schema::build_schema},
    use_case::traits::{
        author::{MockAuthorCommandUseCase, MockAuthorQueryUseCase},
        book::{MockBookCommandUseCase, MockBookQueryUseCase},
        event::MockEventQueryUseCase,
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
        MockEventQueryUseCase::new(),
    );
    let mutation = Mutation::new(
        MockUserCommandUseCase::new(),
        MockBookCommandUseCase::new(),
        MockAuthorCommandUseCase::new(),
    );
    let schema = build_schema(query, mutation);
    println!("{}", schema.sdl());
}
