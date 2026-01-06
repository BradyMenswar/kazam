use crate::KazamClient;

#[allow(async_fn_in_trait)]
pub trait KazamHandler: Send {
    async fn on_challstr(&mut self, client: &mut KazamClient, challstr: &str) {
        let _ = (client, challstr);
    }

    async fn on_update_user(
        &mut self,
        client: &mut KazamClient,
        username: &str,
        named: bool,
        avatar: &str,
    ) {
        let _ = (client, username, named, avatar);
    }

    async fn on_name_taken(&mut self, client: &mut KazamClient, username: &str, message: &str) {
        let _ = (client, username, message);
    }

    async fn on_join(&mut self, client: &mut KazamClient, username: &str, quiet: bool, away: bool) {
        let _ = (client, username, quiet, away);
    }

    async fn on_raw(&mut self, client: &mut KazamClient, room: Option<&str>, content: &str) {
        let _ = (client, room, content);
    }
}
