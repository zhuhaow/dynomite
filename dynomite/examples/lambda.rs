use dynomite::dynamodb::Client as DynamoDbClient;
use lambda_http::handler;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // create aws_sdk_dynamodb client
    let shared_config = aws_config::load_from_env().await;
    let client = DynamoDbClient::new(&shared_config);

    lambda_runtime::run(handler(move |_, _| {
        let client = client.clone();
        async move {
            let tables = client
                .list_tables()
                .send()
                .await?
                .table_names
                .unwrap_or_default();
            Ok::<_, Error>(tables.join("\n"))
        }
    }))
    .await?;

    Ok(())
}
