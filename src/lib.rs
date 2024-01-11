pub mod client;
pub mod confd;
pub mod error;

use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub use client::{Client, ClientBuilder};
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub trait AsyncCallback: Sync + Send {
    fn callback(&self, value: Value) -> Pin<Box<dyn Future<Output = ()> + Sync + Send>>;
}

impl<F, Fut> AsyncCallback for F
where
    F: Fn(Value) -> Fut + Sync + Send + 'static,
    Fut: Future<Output = ()> + Sync + Send + 'static,
{
    fn callback(&self, value: Value) -> Pin<Box<dyn Future<Output = ()> + Sync + Send>> {
        Box::pin((self)(value))
    }
}
