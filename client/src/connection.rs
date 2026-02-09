use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use kazam_protocol::{ServerFrame, parse_server_frame};
use std::time::Duration;

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

pub struct ReconnectPolicy {
    pub max_attempts: Option<usize>,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            max_attempts: Some(5),
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

pub struct Connection {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    url: String,
    reconnect_policy: ReconnectPolicy,
}

impl Connection {
    pub async fn connect(url: String, policy: ReconnectPolicy) -> Result<Self> {
        let ws_stream = Self::establish_connection(&url)
            .await
            .with_context(|| format!("Failed to connect to {}", url))?;

        Ok(Self {
            ws_stream,
            url,
            reconnect_policy: policy,
        })
    }

    async fn establish_connection(url: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let (ws_stream, _) = connect_async(url)
            .await
            .with_context(|| "WebSocket handshake failed")?;
        Ok(ws_stream)
    }

    async fn reconnect(&mut self) -> Result<()> {
        let mut delay = self.reconnect_policy.initial_delay;
        let mut attempt = 1;

        loop {
            if let Some(max) = self.reconnect_policy.max_attempts
                && attempt > max {
                    anyhow::bail!("Failed to reconnect after {} attempts to {}", max, self.url);
                }

            tokio::time::sleep(delay).await;

            match Self::establish_connection(&self.url).await {
                Ok(ws_stream) => {
                    self.ws_stream = ws_stream;
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt,
                        max_attempts = ?self.reconnect_policy.max_attempts,
                        error = %e,
                        "Reconnection attempt failed"
                    );
                    attempt += 1;
                    delay = Duration::from_secs_f64(
                        delay.as_secs_f64() * self.reconnect_policy.backoff_multiplier,
                    )
                    .min(self.reconnect_policy.max_delay);
                }
            }
        }
    }

    pub async fn recv(&mut self) -> Result<ServerFrame> {
        loop {
            match self.ws_stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    return parse_server_frame(&text).context("Failed to parse server frame");
                }
                Some(Ok(Message::Ping(data))) => {
                    self.ws_stream
                        .send(Message::Pong(data))
                        .await
                        .context("Failed to send pong")?;
                }
                Some(Ok(Message::Pong(_))) => continue,
                Some(Ok(Message::Close(_))) | None => {
                    self.reconnect()
                        .await
                        .context("Connection lost and reconnection failed")?;
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => {
                    tracing::error!(error = %e, "WebSocket error, attempting reconnect");
                    self.reconnect()
                        .await
                        .context("WebSocket error and reconnection failed")?;
                }
            }
        }
    }

    pub async fn send(&mut self, message: String) -> Result<()> {
        self.ws_stream
            .send(Message::Text(message))
            .await
            .context("Failed to send message")?;
        Ok(())
    }
}
