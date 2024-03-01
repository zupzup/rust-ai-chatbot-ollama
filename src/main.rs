use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response {
    message: Message,
    done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut messages: Vec<Message> = vec![];
    let client = reqwest::Client::new();

    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    stdout.write_all(b"\n> ").await?;
    stdout.flush().await?;
    while let Some(line) = lines.next_line().await? {
        if !line.is_empty() {
            messages.push(Message {
                role: String::from("user"),
                content: line,
            });

            let mut res = client
                .post("http://127.0.0.1:11434/api/chat")
                .json(&json!({
                    "model": "llama2",
                    "messages": &messages

                }))
                .send()
                .await?;

            let mut message = String::default();
            while let Some(chunk) = res.chunk().await? {
                if let Ok(resp_part) = serde_json::from_slice::<Response>(&chunk) {
                    if !resp_part.done {
                        stdout
                            .write_all(resp_part.message.content.as_bytes())
                            .await?;
                        stdout.flush().await?;

                        message.push_str(&resp_part.message.content);
                    }
                }
            }

            if !message.is_empty() {
                messages.push(Message {
                    role: String::from("assistant"),
                    content: message,
                });
            }
        }

        stdout.write_all(b"\n> ").await?;
        stdout.flush().await?;
    }

    Ok(())
}
