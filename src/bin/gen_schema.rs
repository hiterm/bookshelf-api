use bookshelf_api::{
    presentational::graphql::{mutation::Mutation, query::Query, schema::build_schema},
    use_case::use_case::{mutation::MockMutationUseCase, query::MockQueryUseCase},
};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let query_use_case = MockQueryUseCase::new();
    let query = Query::new(query_use_case);
    let mutation_use_case = MockMutationUseCase::new();
    let mutation = Mutation::new(mutation_use_case);
    let schema = build_schema(query, mutation);
    println!("{}", schema.sdl());
}
