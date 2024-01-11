use confd_client::ClientBuilder;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Events {
    pub enable: bool,
    pub servers: Vec<String>,
}

pub async fn handle_logging(json: serde_json::Value) {
    println!("Got some JSON for logging {}", json);
}

#[tokio::main]
async fn main() -> Result<(), confd_client::Error> {
    env_logger::init();

    // Create the client using the Builder API, subscibing two a couple of
    // handlers.  In one case, we subscribe to an async function.  In the other
    // case, we use a closure with an async body.  Both can be used, however
    // the only limitation here is that only 1 subsciption is allowed per
    // unique configuration path.
    //
    // The call to `.build()` is an async call since it is at that point where
    // the client will connect to the socket and send the initial subscription
    // request.
    let client = ClientBuilder::default()
        .with_subscription(
            "/integration/convoy/events",
            Box::new(|json: serde_json::Value| async move {
                let events: Events =
                    serde_json::from_value(json.get("events").unwrap().clone()).unwrap();
                println!("Got the following: {:?}", events);
            }),
        )
        .with_subscription(
            "/integration/convoy/eventss",
            Box::new(|json: serde_json::Value| async move {
                let events: Events =
                    serde_json::from_value(json.get("events").unwrap().clone()).unwrap();
                println!("Got the following: {:?}", events);
            }),
        )
        .with_subscription("/integration/webtv/logging", Box::new(handle_logging))
        .build()
        .await?;

    // After creating the client, the `listen()` method must be called to react
    // to the configuration changes.  This function is intended to run until
    // one of the socket connections are closed.  It's best then to spawn a
    // background task using `tokio::spawn()` to run this function, but for
    // simplicity in the example, we can simly call it here since we have nothing
    // else to do in the background.
    client.listen().await?;

    Ok(())
}
