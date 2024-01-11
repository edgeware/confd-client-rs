use confd_client::Client;

async fn handler(json: serde_json::Value) {
    println!("Got JSON {}", json);
}

#[tokio::main]
async fn main() -> Result<(), confd_client::Error> {
    let mut client = Client::new("function-example", "/var/confd/service-interface.socket");

    client
        .subscribe("/storage/alarms", Box::new(handler))
        .await?;

    client.listen().await?;

    Ok(())
}
