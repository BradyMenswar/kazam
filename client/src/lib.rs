use std::collections::HashMap;

use crate::connection::{Connection, ReconnectPolicy};
use anyhow::Result;
use kazam_protocol::{ServerFrame, ServerMessage};

mod auth;
mod connection;
mod handler;
mod room;

pub use handler::KazamHandler;
pub use room::{RoomId, RoomState};

pub const SHOWDOWN_URL: &str = "wss://sim3.psim.us/showdown/websocket";
pub struct KazamClient {
    connection: Connection,
    rooms: HashMap<RoomId, RoomState>,
}

impl KazamClient {
    pub async fn init(url: &str) -> Result<Self> {
        let connection = Connection::connect(url.to_string(), ReconnectPolicy::default()).await?;
        Ok(Self {
            connection,
            rooms: HashMap::new(),
        })
    }

    pub async fn run<H: KazamHandler>(&mut self, handler: &mut H) -> Result<()> {
        loop {
            let frame = self.connection.recv().await?;
            self.dispatch_frame(frame, handler).await?;
        }
    }

    pub async fn send_raw(&mut self, message: String) -> Result<()> {
        self.connection.send(message).await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.connection.close().await?;
        Ok(())
    }

    async fn dispatch_frame<H: KazamHandler>(
        &mut self,
        frame: ServerFrame,
        handler: &mut H,
    ) -> Result<()> {
        for message in frame.messages {
            match message {
                ServerMessage::Challstr(challstr) => {
                    handler.on_challstr(self, &challstr).await;
                }

                ServerMessage::UpdateUser {
                    username,
                    named,
                    avatar,
                } => {
                    handler
                        .on_update_user(self, &username, named, &avatar)
                        .await;
                }

                ServerMessage::NameTaken { username, message } => {
                    handler.on_name_taken(self, &username, &message).await;
                }

                ServerMessage::Join {
                    username,
                    quiet,
                    away,
                } => {
                    handler.on_join(self, &username, quiet, away).await;
                }

                ServerMessage::Raw(content) => {
                    handler
                        .on_raw(self, frame.room_id.as_deref(), &content)
                        .await
                }
            }
        }
        Ok(())
    }
}
