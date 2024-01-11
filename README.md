# Edgeware Confd Client for Rust

This is a client library for subscribing to configuration changes using Confd
for Rust.

## Installation

Add the following to your `cargo.toml` file

```toml
[dependencies]
confd-client = { git = "https://github.com/edgeware/confd-client-rs.git", branch="master" }
```

### From the rust-lang book in the chapter on specifying dependencies

> Anything that is not a branch or tag falls under rev. This can be a commit hash like
> rev = "4c59b707", or a named reference exposed by the remote repository such as rev
> = "refs/pull/493/head". What references are available varies by where the repo is
> hosted; GitHub in particular exposes a reference to the most recent commit of every
> pull request as shown, but other git hosts often provide something equivalent,
> possibly under a different naming scheme.
>
> Once a git dependency has been added, Cargo will lock that dependency to the latest
> commit at the time. New commits will not be pulled down automatically once the lock
> is in place. However, they can be pulled down manually with cargo update.

## Usage

The following example shows how to subscribe to the `integration.convoy.events` path
and simply print out the configuration on changes.  Each time the config is modified
in `confd` the subscribed function will be called.

```rust
use confd_client::ClientBuilder;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Events {
    pub enable: bool,
    pub servers: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), confd_client::Error> {
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

    client.listen().await?;
    Ok(())
}
```

In addition to a closure, an `async` function can be used as thee subscription.
For example:

```rust
use confd_client::ClientBuilder;
use serde::Deserialize;

// ...
async fn handle_convoy_events(json: serde_json::Value) {
    let events: Events = serde_json::from_value(json.get("events").unwrap().clone()).unwrap();
    println!("Got the following: {:?}", events);
}

#[tokio::main]
async fn main() -> Result<(), confd_client::Error> {
   let client = ClientBuilder::default()
        .with_subscription(
            "/integration/convoy/events",
            Box::new(handle_convoy_events),
        )
        .build()
        .await?;

    client.listen().await?;
    Ok(())
}
```

Additional examples are located in the `/examples` subfolder of this repository.

It is recommended to call `listen()` from within a spawned task, since that function is
designed to run for the lifetime of the program.

E.g.

```rust
let listener = tokio::spawn(client.listen());
let _ = tokio::join!(listener);
```
