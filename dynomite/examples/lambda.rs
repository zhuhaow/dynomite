use dynomite::dynamodb::Client as DynamoDbClient;
use lambda_http::service_fn;
use lambda_runtime::LambdaEvent;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // create aws_sdk_dynamodb client
    let shared_config = aws_config::load_from_env().await;
    let client = DynamoDbClient::new(&shared_config);

    lambda_runtime::run(service_fn(move |_: LambdaEvent<usize>| {
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
