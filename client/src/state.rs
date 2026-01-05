/// Information about the currently logged-in user
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub logged_in: bool,
    pub avatar: String,
}

/// Internal state accumulated from messages
pub(crate) struct ClientState {
    pub challstr: Option<String>,
    pub user: Option<UserInfo>,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            challstr: None,
            user: None,
        }
    }
}
