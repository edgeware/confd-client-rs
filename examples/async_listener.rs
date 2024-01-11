use confd_client::Client;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Events {
    pub enable: bool,
    pub servers: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), confd_client::Error> {
    env_logger::init();

    let mut client = Client::new("closure-example", "/var/confd/service-interface.socket");

    client
        .subscribe(
            "/integration/convoy/events",
            Box::new(|json: serde_json::Value| async move {
                let events: Events =
                    serde_json::from_value(json.get("events").unwrap().clone()).unwrap();
                println!("Got the following: {:?}", events);
            }),
        )
        .await?;

    let listener = tokio::spawn(client.listen());

    let _ = tokio::join!(listener);

    Ok(())
}
