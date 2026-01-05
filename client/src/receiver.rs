use anyhow::Result;
use tokio::sync::mpsc;

use crate::handler::Handler;
use crate::state::{ClientState, UserInfo};
use kazam_protocol::{ServerFrame, ServerMessage};

/// Receives messages from the server and dispatches them to a handler.
pub struct Receiver {
    incoming: mpsc::Receiver<Result<ServerFrame>>,
    state: ClientState,
}

impl Receiver {
    pub(crate) fn new(incoming: mpsc::Receiver<Result<ServerFrame>>, state: ClientState) -> Self {
        Self { incoming, state }
    }

    /// Run the message loop, dispatching events to the handler.
    ///
    /// This will run until the connection is closed or an error occurs.
    pub async fn run<H: Handler>(&mut self, handler: &mut H) -> Result<()> {
        while let Some(frame_result) = self.incoming.recv().await {
            let frame = frame_result?;
            self.dispatch_frame(handler, frame).await;
        }
        Ok(())
    }

    /// Dispatch a single frame to the handler
    async fn dispatch_frame<H: Handler>(&mut self, handler: &mut H, frame: ServerFrame) {
        let room = frame.room_id.as_deref();

        for msg in frame.messages {
            self.update_state(&msg);
            self.dispatch_message(handler, room, msg).await;
        }
    }

    /// Update internal state based on a message
    fn update_state(&mut self, msg: &ServerMessage) {
        match msg {
            ServerMessage::Challstr(challstr) => {
                self.state.challstr = Some(challstr.clone());
            }
            ServerMessage::UpdateUser {
                username,
                named,
                avatar,
            } => {
                self.state.user = Some(UserInfo {
                    username: username.clone(),
                    logged_in: *named,
                    avatar: avatar.clone(),
                });
            }
            _ => {}
        }
    }

    /// Dispatch a single message to the appropriate handler method
    async fn dispatch_message<H: Handler>(
        &self,
        handler: &mut H,
        room: Option<&str>,
        msg: ServerMessage,
    ) {
        match msg {
            ServerMessage::Challstr(challstr) => {
                handler.on_challstr(&challstr).await;
            }
            ServerMessage::UpdateUser {
                username,
                named,
                avatar,
            } => {
                handler.on_update_user(&username, named, &avatar).await;
            }
            ServerMessage::NameTaken { username, message } => {
                handler.on_name_taken(&username, &message).await;
            }
            ServerMessage::Raw(content) => {
                handler.on_raw(room, &content).await;
            }
        }
    }

    /// Get the stored challstr, if any
    pub fn challstr(&self) -> Option<&str> {
        self.state.challstr.as_deref()
    }

    /// Get current user info
    pub fn user(&self) -> Option<&UserInfo> {
        self.state.user.as_ref()
    }
}
