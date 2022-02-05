use aws_sdk_dynamodb::model::{KeyType, ScalarAttributeType};
/// Assumes a you are running the following `dynamodb-local`
/// on your host machine
///
/// ```bash
/// $ docker run -p 8000:8000 amazon/dynamodb-local
/// ```
use dynomite::{
    dynamodb::{
        model::{AttributeDefinition, KeySchemaElement, ProvisionedThroughput},
        Client as DynamoDbClient,
    },
    Attribute, Item,
};
use maplit::hashmap;
use std::{convert::TryFrom, error::Error};
use uuid::Uuid;

#[derive(Item, Debug, Clone)]
pub struct Book {
    #[dynomite(partition_key, rename = "Id")]
    id: Uuid,
    #[dynomite(rename = "bookTitle", default)]
    title: String,
}

/// create a book table with a single string (S) primary key.
/// if this table does not already exists
/// this may take a second or two to provision.
/// it will fail if this table already exists but that's okay,
/// this is just an example :)
async fn bootstrap(
    client: &DynamoDbClient,
    table_name: String,
) {
    let _ = client
        .create_table()
        .table_name(table_name)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("Id")
                .key_type(KeyType::Hash)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("Id")
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(1)
                .write_capacity_units(1)
                .build(),
        )
        .send()
        .await;
}

// this will create a rust book shelf in your aws account!
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    // create aws_sdk_dynamodb client
    let shared_config = aws_config::load_from_env().await;
    let client = DynamoDbClient::new(&shared_config);

    let table_name = "books".to_string();

    bootstrap(&client, table_name.clone()).await;

    let book = Book {
        id: Uuid::new_v4(),
        title: "rust".into(),
    };

    // print the key for this book
    // requires bringing `dynomite::Item` into scope
    println!("book.key() {:#?}", book.key());

    // add a book to the shelf
    println!(
        "put_item() result {:#?}",
        client
            .put_item()
            .table_name(&table_name)
            .set_item(Some(book.clone().into()))
            .send()
            .await?
    );

    println!(
        "put_item() result {:#?}",
        client
            .put_item()
            .table_name(&table_name)
            .item("Id", Uuid::new_v4().to_string().into_attr())
            .item("bookTitle", "rust and beyond".to_string().into_attr())
            .send()
            .await?
    );

    // scan through all pages of results in the books table for books who's title is "rust"
    println!(
        "scan result {:#?}",
        client
            .clone()
            .scan()
            .table_name(&table_name)
            .filter_expression("bookTitle = :title")
            .set_expression_attribute_values(Some(hashmap!(
                ":title".to_string() => "rust".to_string().into_attr()
            )))
            .send()
            .await?
    );

    // get the "rust' book by the Book type's generated key
    println!(
        "get_item() result {:#?}",
        client
            .get_item()
            .table_name(table_name)
            .key("Id", book.id.to_string().into_attr())
            .send()
            .await?
            .item
            .map(Book::try_from) // attempt to convert a attribute map to a book type
    );

    Ok(())
}
