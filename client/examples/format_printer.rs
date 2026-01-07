use std::env;
use std::process;

use anyhow::Result;
use kazam_client::{FormatSection, KazamClient, KazamHandler, SHOWDOWN_URL};

struct FormatPrinter;

impl KazamHandler for FormatPrinter {
    async fn on_formats(&mut self, sections: &[FormatSection]) {
        println!("\n=== Available Formats ===\n");

        for section in sections {
            println!("┌─ {} (Column {})", section.name, section.column);
            println!("│");

            for format in &section.formats {
                let mut flags = Vec::new();
                if format.random_team {
                    flags.push("random");
                }
                if format.search_show {
                    flags.push("ladder");
                }
                if format.challenge_show {
                    flags.push("challenge");
                }
                if format.tournament_show {
                    flags.push("tournament");
                }
                if format.level_50 {
                    flags.push("lv50");
                }
                if format.best_of {
                    flags.push("bo3");
                }
                if format.tera_preview {
                    flags.push("tera");
                }

                let flag_str = if flags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", flags.join(", "))
                };

                println!("│  • {}{}", format.name, flag_str);
            }

            println!("│");
            println!("└─ {} formats\n", section.formats.len());
        }

        let total: usize = sections.iter().map(|s| s.formats.len()).sum();
        println!("Total: {} formats in {} sections", total, sections.len());

        process::exit(0);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = env::args().nth(1).unwrap_or_else(|| SHOWDOWN_URL.to_string());

    println!("Connecting to {}...", url);
    let mut client = KazamClient::connect(&url).await?;
    println!("Connected. Waiting for formats...\n");

    let mut handler = FormatPrinter;
    client.run(&mut handler).await
}
