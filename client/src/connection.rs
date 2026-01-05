use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use kazam_protocol::{ServerFrame, parse_server_frame};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

const CHANNEL_BUFFER_SIZE: usize = 64;

pub struct Connection {
    incoming: mpsc::Receiver<Result<ServerFrame>>,
    outgoing: mpsc::Sender<String>,
}

impl Connection {
    /// Connect to a WebSocket server
    pub async fn connect(url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(url).await?;
        let (write, read) = ws_stream.split();

        let (incoming_tx, incoming_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        let (outgoing_tx, outgoing_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

        tokio::spawn(Self::read_task(read, incoming_tx));
        tokio::spawn(Self::write_task(write, outgoing_rx));

        Ok(Self {
            incoming: incoming_rx,
            outgoing: outgoing_tx,
        })
    }

    /// Read task - reads from WebSocket and sends to incoming channel
    async fn read_task<S>(mut read: S, tx: mpsc::Sender<Result<ServerFrame>>)
    where
        S: StreamExt<Item = Result<WsMessage, tokio_tungstenite::tungstenite::Error>> + Unpin,
    {
        while let Some(msg_result) = read.next().await {
            let result = match msg_result {
                Ok(WsMessage::Text(text)) => match parse_server_frame(&text) {
                    Ok(frame) => Ok(frame),
                    Err(e) => Err(e),
                },
                Ok(WsMessage::Close(_)) => break,
                Ok(_) => continue, // Ignore ping/pong/binary
                Err(e) => Err(e.into()),
            };

            if tx.send(result).await.is_err() {
                // Receiver dropped, exit
                break;
            }
        }
    }

    /// Write task - reads from outgoing channel and writes to WebSocket
    async fn write_task<S>(mut write: S, mut rx: mpsc::Receiver<String>)
    where
        S: SinkExt<WsMessage> + Unpin,
        S::Error: std::fmt::Debug,
    {
        while let Some(msg) = rx.recv().await {
            if write.send(WsMessage::Text(msg.into())).await.is_err() {
                break;
            }
        }
    }

    /// Split the connection into its incoming and outgoing channels
    pub fn split(self) -> (mpsc::Receiver<Result<ServerFrame>>, mpsc::Sender<String>) {
        (self.incoming, self.outgoing)
    }
}
