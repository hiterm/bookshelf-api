use bookshelf_api::dependency_injection::dependency_injection;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let schema = dependency_injection().await;
    println!("{}", schema.sdl());
}
