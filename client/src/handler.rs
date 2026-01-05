use async_trait::async_trait;

/// Trait for handling Pokemon Showdown server messages.
///
/// Implement this trait to create a message handler. All methods have
/// default no-op implementations, so you only need to implement the
/// events you care about.
///
/// # Example
///
/// ```ignore
/// struct MyBot {
///     sender: Sender,
/// }
///
/// #[async_trait]
/// impl Handler for MyBot {
///     async fn on_challstr(&mut self, challstr: &str) {
///         self.sender.login("user", "pass", challstr).await.ok();
///     }
/// }
/// ```
#[async_trait]
pub trait Handler: Send {
    /// Called when the server sends a challenge string.
    /// This indicates that login is now possible.
    async fn on_challstr(&mut self, challstr: &str) {
        let _ = challstr;
    }

    /// Called when user info is updated (after login or name change).
    async fn on_update_user(&mut self, username: &str, logged_in: bool, avatar: &str) {
        let _ = (username, logged_in, avatar);
    }

    /// Called when a login attempt fails.
    async fn on_name_taken(&mut self, username: &str, message: &str) {
        let _ = (username, message);
    }

    /// Called for any message not handled by a specific method.
    /// The room parameter is Some if the message was sent in a room context.
    async fn on_raw(&mut self, room: Option<&str>, content: &str) {
        let _ = (room, content);
    }
}
