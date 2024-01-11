use confd_client::ClientBuilder;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Events {
    pub enable: bool,
    pub servers: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), confd_client::Error> {
    env_logger::init();

    let client = ClientBuilder::default()
        .with_subscription(
            "/integration/convoy/events",
            Box::new(|json: serde_json::Value| async move {
                let events: Events =
                    serde_json::from_value(json.get("events").unwrap().clone()).unwrap();
                println!("Got the following: {:?}", events);
            }),
        )
        .build()
        .await?;

    let listener = tokio::spawn(async move {
        client.listen().await.unwrap_or_else(|err| {
            println!("Received error while listening for events: {}", err);
        });
    });

    let _ = tokio::join!(listener);

    Ok(())
}
