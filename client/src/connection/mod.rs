use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use kazam_protocol::{ClientMessage, ServerFrame, parse_server_frame};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Low-level WebSocket connection handler
pub struct Connection {
    ws: WsStream,
}

impl Connection {
    /// Connect to a WebSocket URL
    pub async fn connect(url: &str) -> Result<Self> {
        let (ws, _response) = connect_async(url)
            .await
            .context("Failed to connect to WebSocket")?;

        Ok(Self { ws })
    }

    /// Receive the next WebSocket frame for the server
    pub async fn next_frame(&mut self) -> Result<Option<ServerFrame>> {
        while let Some(message) = self.ws.next().await {
            let message = message.context("WebSocket error")?;

            match message {
                Message::Text(text) => {
                    let frame =
                        parse_server_frame(&text).context("Failed to parse server frame")?;
                    return Ok(Some(frame));
                }
                Message::Close(_) => return Ok(None),
                Message::Ping(data) => self.ws.send(Message::Pong(data)).await?,
                _ => {}
            }
        }

        Ok(None)
    }

    /// Send a client message
    pub async fn send(&mut self, message: ClientMessage) -> Result<()> {
        let wire_format = message.to_wire_format();
        self.ws
            .send(Message::Text(wire_format))
            .await
            .context("Failed to send message")
    }

    /// Send a raw string
    pub async fn send_raw(&mut self, text: &str) -> Result<()> {
        self.ws
            .send(Message::Text(text.into()))
            .await
            .context("Failed to send raw message")
    }
}
