mod auth;
mod connection;

use anyhow::{Ok, Result};
use kazam_protocol::{ClientMessage, ServerFrame};

use auth::AuthState;
pub use connection::Connection;

/// Main client for interacting with Pokemon Showdown
pub struct Client {
    connection: Connection,
    auth_state: AuthState,
}

impl Client {
    /// Connect to the default Pokemon Showdown server
    pub async fn connect_default() -> Result<Self> {
        Self::connect("wss://sim3.psim.us/showdown/websocket").await
    }

    /// Connect to a specific Pokemon Showdown server
    pub async fn connect(url: &str) -> Result<Self> {
        let connection = Connection::connect(url).await?;
        Ok(Self {
            connection,
            auth_state: AuthState::Connected,
        })
    }

    /// Get current authentication state
    pub fn auth_state(&self) -> &AuthState {
        &self.auth_state
    }

    /// Get username if authenticated
    pub fn username(&self) -> Option<&str> {
        match &self.auth_state {
            AuthState::Guest { username } => Some(username),
            AuthState::Authenticated { username } => Some(username),
            AuthState::Connected => None,
        }
    }

    /// Login with username and password
    pub async fn login_with_password(&mut self, username: &str, password: &str) -> Result<String> {
        let confirmed = auth::login_with_password(&mut self.connection, username, password).await?;
        self.auth_state = AuthState::Authenticated {
            username: confirmed.clone(),
        };
        Ok(confirmed)
    }

    /// Receive next websocket frame from server
    pub async fn next_frame(&mut self) -> Result<Option<ServerFrame>> {
        self.connection.next_frame().await
    }

    /// Send a message to the server
    pub async fn send(&mut self, message: ClientMessage) -> Result<()> {
        self.connection.send(message).await
    }

    /// Send a raw string
    pub async fn send_raw(&mut self, text: &str) -> Result<()> {
        self.connection.send_raw(text).await
    }
}
