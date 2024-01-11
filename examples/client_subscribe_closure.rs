use confd_client::Client;

#[tokio::main]
async fn main() -> Result<(), confd_client::Error> {
    let mut client = Client::new("closure-example", "/var/confd/service-interface.socket");

    client
        .subscribe(
            "/integration/convoy/events",
            Box::new(|json| async move {
                println!("Got the following: {}", json);
            }),
        )
        .await?;

    client.listen().await?;

    Ok(())
}
