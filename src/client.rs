use crate::confd;
use crate::AsyncCallback;
use crate::Result;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::net::UnixStream;

struct SocketCallback {
    socket: UnixStream,
    callback: Box<dyn AsyncCallback + Sync + Send + 'static>,
}

pub struct Client {
    client_name: String,
    socket_path: PathBuf,
    subscribers: BTreeMap<PathBuf, SocketCallback>,
}

impl Client {
    /// Create a new `confd` client.
    #[must_use]
    pub fn new<P: AsRef<Path>>(client_name: &str, socket_path: P) -> Client {
        Self {
            client_name: client_name.to_string(),
            socket_path: socket_path.as_ref().to_owned(),
            subscribers: BTreeMap::new(),
        }
    }

    /// Register an callback for a given configuraiton path.
    ///
    /// While this function may be called multiple times for a given
    /// client, however only one callback will be registered for a given Path.
    ///
    /// Each callback will be called once immediately to get the initial
    /// configuraiton, and then again any time the configuration changes.
    ///
    /// # Errors
    ///
    /// This function will return an `Error` if either there is a problem
    /// connecting to the socket or if the subscription request cannot be
    /// written to the socket.
    pub async fn subscribe<P: AsRef<Path>>(
        &mut self,
        path: P,
        callback: Box<dyn AsyncCallback + Sync + Send + 'static>,
    ) -> Result<()>
    where
        P: AsRef<Path>,
    {
        log::debug!("Subscribing to {}", path.as_ref().display());
        let path = path.as_ref().to_owned();

        let subscription_req = serde_json::to_string(&json!({
            "path": path.clone(),
            "subscriber": self.client_name,
        }))
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

        log::debug!(
            "Connecting to socket {}",
            self.socket_path.to_string_lossy()
        );
        let mut socket = UnixStream::connect(&self.socket_path).await?;

        log::debug!("Sending subscription request {}", subscription_req);
        confd::send(&mut socket, subscription_req).await?;

        self.subscribers
            .insert(path, SocketCallback { socket, callback });
        Ok(())
    }

    /// Listen for configuration changes, and notify the subscribers on updates.
    ///
    /// # Errors
    ///
    /// This function will return an `Error` if there is a problem reading from
    /// the sockets or if the message can not be parsed as valid JSON.
    pub async fn listen(self) -> Result<()> {
        log::debug!(
            "Awaiting configuration changes for {} paths",
            self.subscribers.len()
        );

        let tasks = self
            .subscribers
            .into_iter()
            .map(|(path, mut socket_callback)| {
                tokio::spawn(async move {
                    loop {
                        match confd::read(&mut socket_callback.socket).await {
                            Ok(message) => {
                                log::debug!("Received message: {}", message);
                                if let serde_json::Value::Object(_) = &message {
                                    let callback = &socket_callback.callback;

                                    log::debug!(
                                        "Executing callback for path {}",
                                        path.to_string_lossy().to_string()
                                    );
                                    callback.callback(message.clone()).await;
                                }
                            }
                            Err(err) => {
                                log::error!("Error reading from socket: {}", err);
                                break;
                            }
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        tokio::join!(futures::future::join_all(tasks));

        Ok(())
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct ClientBuilder {
    name: String,
    socket_path: PathBuf,
    subscribers: BTreeMap<PathBuf, Box<dyn AsyncCallback + Sync + Send + 'static>>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            name: String::from("confd-rs-client"),
            socket_path: PathBuf::from("/var/confd/service-interface.socket"),
            subscribers: BTreeMap::default(),
        }
    }
}

impl ClientBuilder {
    /// Set the socket path.
    #[must_use]
    pub fn with_socket_path(mut self, path: PathBuf) -> Self {
        self.socket_path = path.clone();

        self
    }

    /// Set the client name.
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = String::from(name);

        self
    }

    /// Register a callback function as a subscription for the given path.
    #[must_use]
    pub fn with_subscription<P: AsRef<Path>>(
        mut self,
        path: P,
        callback: Box<dyn AsyncCallback + Sync + Send + 'static>,
    ) -> Self {
        self.subscribers.insert(path.as_ref().to_owned(), callback);

        self
    }

    /// Build a new `confd` client.
    ///
    /// This function attempts to subscribe to all subscriptions, which
    /// will open a connection on the socket.
    ///
    /// # Errors
    ///
    /// This function will return an `Error` if any of the subscription
    /// requests fail.  The most likely reason for this is that the
    /// we are unable to write to the socket.
    pub async fn build(self) -> Result<Client> {
        let mut client = Client::new(&self.name, &self.socket_path);

        for (path, callback) in self.subscribers {
            client.subscribe(path.clone(), callback).await?;
        }

        Ok(client)
    }
}
