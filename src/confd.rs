use crate::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Send a message using the confd protocol.
///
/// # Errors
///
/// This function will return an `Error` if there is a problem reading the
/// header information from the Confd socket, or if there is a problem writing
/// either the header or the subscription request payload to to socket.
pub async fn send(socket: &mut UnixStream, message: String) -> Result<()> {
    let payload = message.as_bytes();
    let header = u32::try_from(payload.len())?.to_be_bytes();

    log::debug!("Send {} bytes", payload.len());
    log::trace!("payload: {:?}", payload);

    socket.write_all(&header).await?;
    socket.write_all(payload).await?;
    socket.flush().await?;

    Ok(())
}

/// Read a JSON message using the confd protocol.
///
/// # Errors
/// This function will return an error if there is a problem reading from
/// the socket or if the message is not valid JSON.
pub async fn read(socket: &mut UnixStream) -> Result<serde_json::Value> {
    let mut header = [0u8; 4];
    socket.read_exact(&mut header).await?;

    let payload_len = u32::from_be_bytes(header);
    let mut payload = vec![0u8; payload_len as usize];

    log::debug!("Read {} bytes", payload_len);

    socket.read_exact(&mut payload).await?;
    log::trace!("received: {:?}", payload);

    Ok(serde_json::from_slice(&payload)?)
}
