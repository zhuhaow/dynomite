use dynomite::{
    dynamodb::{
        model::{
            AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput,
            ScalarAttributeType,
        },
        Client as DynamoDbClient,
    },
    Attribute, Attributes, Item,
};
use maplit::hashmap;
use std::{convert::TryFrom, error::Error};
use uuid::Uuid;

#[derive(Attributes, Debug, Clone)]
pub struct Author {
    id: Uuid,
    #[dynomite(default)]
    name: String,
}

#[derive(Item, Debug, Clone)]
pub struct Book {
    #[dynomite(partition_key)]
    id: Uuid,
    #[dynomite(rename = "bookTitle")]
    title: String,
    authors: Option<Vec<Author>>,
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
        .send();
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

    let authors = Some(vec![Author {
        id: Uuid::new_v4(),
        name: "Jo Bloggs".into(),
    }]);

    let book = Book {
        id: Uuid::new_v4(),
        title: "rust".into(),
        authors,
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
            .set_item(Some(book.clone().into())) // <= convert book into it's attribute map representation
            .send()
            .await?
    );

    println!(
        "put_item() result {:#?}",
        client
            .put_item()
            .table_name(&table_name)
            .set_item(Some(
                Book {
                    id: Uuid::new_v4(),
                    title: "rust and beyond".into(),
                    authors: Some(vec![Author {
                        id: Uuid::new_v4(),
                        name: "Jim Ferris".into(),
                    }]),
                }
                .into()
            ))
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
            .await? // attempt to convert a attribute map to a book type
    );

    // get the "rust' book by the Book type's generated key
    println!(
        "get_item() result {:#?}",
        client
            .get_item()
            .table_name(&table_name)
            .key(&book.id.to_string(), book.title.into_attr())
            .send()
            .await?
            .item
            .map(Book::try_from) // attempt to convert a attribute map to a book type
    );
    Ok(())
}
