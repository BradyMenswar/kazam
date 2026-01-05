use anyhow::Result;
use kazam_client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect_default().await?;

    println!("Connected. Waiting for messages...");

    loop {
        if let Some(frame) = client.next_frame().await? {
            println!("\n=== Frame ===");
            println!("Room: {:?}", frame.room_id);
            println!("Messages: {:?}", frame.messages);
        }
    }
}
