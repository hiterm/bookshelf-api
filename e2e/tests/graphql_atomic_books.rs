use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn atomic_book_conflict_and_update_contract() -> Result<()> {
    let (_, token) = create_test_user().await?;
    let fields = r#"title: "Atomic", authorIds: [], isbn: "", read: false, owned: false, priority: 50, format: UNKNOWN, store: UNKNOWN"#;
    let create = |names: &str| {
        format!(
            "mutation {{ createBook(bookData: {{ {fields}, newAuthorNames: {names} }}) {{ book {{ id authors {{ id name }} }} operationId revisionNumber }} }}"
        )
    };
    let (_, first) = graphql_request(&create(r#"["Z"]"#), Some(&token)).await?;
    assert!(first.get("errors").is_none(), "{first}");
    let id = first["data"]["createBook"]["book"]["id"]
        .as_str()
        .context("book id")?;
    let (_, failed) = graphql_request(&create(r#"["A", "Z"]"#), Some(&token)).await?;
    assert_eq!(failed["errors"][0]["extensions"]["code"], "CONFLICT");
    assert_eq!(
        failed["errors"][0]["extensions"]["reason"],
        "AUTHOR_NAME_CONFLICT"
    );
    let update = format!(
        "mutation {{ updateBook(bookData: {{ id: \"{id}\", {fields}, newAuthorNames: [\"A\", \"Z\"] }}) {{ book {{ id }} }} }}"
    );
    let (_, failed) = graphql_request(&update, Some(&token)).await?;
    assert_eq!(
        failed["errors"][0]["extensions"]["reason"],
        "AUTHOR_NAME_CONFLICT"
    );
    let (_, state) = graphql_request(
        "{ authors { name } books { id } operations { id } }",
        Some(&token),
    )
    .await?;
    assert_eq!(
        state["data"]["authors"]
            .as_array()
            .context("authors")?
            .len(),
        1
    );
    assert_eq!(state["data"]["books"].as_array().context("books")?.len(), 1);
    assert_eq!(
        state["data"]["operations"]
            .as_array()
            .context("operations")?
            .len(),
        1
    );
    let update = update.replace(r#"["A", "Z"]"#, r#"["A", "B"]"#);
    let (_, saved) = graphql_request(&update, Some(&token)).await?;
    assert!(saved.get("errors").is_none(), "{saved}");
    Ok(())
}
