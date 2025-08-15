use anyhow::Result;
use dotenv::dotenv;
use pubky::{Client, Keypair};
use pubky_app_specs::{PubkyAppPost, PubkyAppPostKind, PubkyAppUser};
use pubky_timestamp::Timestamp;
use std::env;
use bip39::Mnemonic;
use serde::{Deserialize, Serialize};
use reqwest;
use std::fs;

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Notification {
    timestamp: i64,
    body: NotificationBody,
}

#[derive(Debug, Serialize, Deserialize)]
struct NotificationBody {
    #[serde(rename = "type")]
    notification_type: String,
    #[serde(default)]
    mentioned_by: Option<String>,
    #[serde(default)]
    post_uri: Option<String>,
    #[serde(default)]
    followed_by: Option<String>,
    #[serde(default)]
    tagged_by: Option<String>,
    #[serde(default)]
    tag_label: Option<String>,
    #[serde(default)]
    replied_by: Option<String>,
    #[serde(default)]
    parent_post_uri: Option<String>,
    #[serde(default)]
    reply_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LastRead {
    timestamp: i64,
}

async fn get_post_content(client: &Client, post_uri: &str) -> Result<String> {
    println!("Fetching post content from: {}", post_uri);
    let response = client.get(post_uri).send().await?;
    let status = response.status();
    println!("Post response status: {}", status);
    
    let body = response.bytes().await?;
    println!("Post response body length: {} bytes", body.len());
    
    if body.is_empty() {
        println!("Warning: Received empty post response");
        return Ok("".to_string());
    }
    
    // Try to parse as PubkyAppPost first
    match serde_json::from_slice::<PubkyAppPost>(&body) {
        Ok(post) => {
            println!("Successfully parsed post as PubkyAppPost");
            return Ok(post.content);
        }
        Err(e) => {
            println!("Failed to parse as PubkyAppPost: {}", e);
            // If it's just a string, return it directly
            if let Ok(content) = String::from_utf8(body.to_vec()) {
                println!("Successfully parsed post as plain text");
                return Ok(content);
            }
            return Err(anyhow::anyhow!("Failed to parse post content: {}", e));
        }
    }
}

async fn read_knowledge_base() -> Result<String> {
    let content = fs::read_to_string("knowledge-base.txt")?;
    Ok(content)
}

async fn generate_response(content: &str) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY").map_err(|_| anyhow::anyhow!("OPENAI_API_KEY not found in .env"))?;
    let client = reqwest::Client::new();
    
    let knowledge_base = read_knowledge_base().await?;
    
    let request = ChatRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: format!("You are a friendly and knowledgeable AI assistant that can discuss any topic. You have deep knowledge about Pubky, a decentralized social media platform, but you are not limited to just that. You can engage in conversations about any subject while maintaining a helpful and informative tone. You must respond in English by default, but if the user's post is in another language, your response should also be in that language.

IMPORTANT RULES:
1. Your responses MUST be exactly 1000 characters or less. This is a strict limit.
2. Write in a natural, conversational style. Avoid numbered lists or bullet points.
3. Make sure your response is complete and well-formed. Never end mid-sentence or mid-thought.
4. If you need to be concise, focus on the most important points and express them clearly.
5. Maintain a friendly and engaging tone throughout your response.

Here is the knowledge base about Pubky that you can reference when needed:\n\n{}", knowledge_base),
            },
            ChatMessage {
                role: "user".to_string(),
                content: content.to_string(),
            },
        ],
        temperature: 0.7,
        max_tokens: 250,
    };

    println!("Sending request to OpenAI API...");
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let status = response.status();
    println!("OpenAI API response status: {}", status);
    
    let response_text = response.text().await?;
    println!("OpenAI API response body: {}", response_text);
    
    let chat_response: ChatResponse = serde_json::from_str(&response_text)?;
    let content = chat_response.choices[0].message.content.clone();
    
    // Double check the length and log it
    println!("Response length: {} characters", content.len());
    if content.len() > 1000 {
        println!("Warning: Response exceeded 1000 characters despite instructions!");
        return Ok(content[..1000].to_string());
    }
    
    Ok(content)
}

async fn load_or_create_keypair() -> Result<Keypair> {
    let secret_words = env::var("BOT_SECRET_KEY").map_err(|_| anyhow::anyhow!("BOT_SECRET_KEY not found in .env"))?;
    let mnemonic = Mnemonic::parse_normalized(&secret_words)?;
    let seed_bytes = mnemonic.to_seed("");
    let secret_array: [u8; 32] = seed_bytes[..32].try_into().map_err(|_| anyhow::anyhow!("Invalid seed"))?;
    let keypair = Keypair::from_secret_key(&secret_array);
    
    let public_key = env::var("BOT_PUBLIC_KEY").map_err(|_| anyhow::anyhow!("BOT_PUBLIC_KEY not found in .env"))?;
    if keypair.public_key().to_string() != public_key {
        return Err(anyhow::anyhow!("Public key does not match seed"));
    }
    
    Ok(keypair)
}

async fn setup_client() -> Result<(Client, Keypair)> {
    dotenv().ok();
    println!("Environment variables loaded from .env");

    let is_testnet = env::var("TESTNET").map(|v| v == "true").unwrap_or(false);
    let client = if is_testnet {
        println!("Using testnet configuration");
        Client::builder().testnet().build()?
    } else {
        println!("Using mainnet configuration");
        Client::builder().build()?
    };
    
    let keypair = load_or_create_keypair().await?;

    match client.signin(&keypair).await {
        Ok(_) => println!("Signin successful!"),
        Err(e) => {
            println!("Signin failed: {}", e);
            return Err(anyhow::anyhow!("Failed to signin: {}", e));
        }
    }

    Ok((client, keypair))
}

async fn create_profile(client: &Client, keypair: &Keypair) -> Result<()> {
    let profile = PubkyAppUser {
        name: "AI Rand".to_string(),
        bio: Some("Mention me and I will respond to you!".to_string()),
        image: Some("pubky://338pqgzxks8hhqzs7ucfwn17w4qujcfgh58onn6dakwk3r9hxy5o/pub/pubky.app/files/003331KGWWCE0".to_string()),
        links: None,
        status: None,
    };

    let profile_json = serde_json::to_string(&profile)?;
    let url = format!("pubky://{}/pub/pubky.app/profile.json", keypair.public_key());
    
    client.put(&url)
        .body(profile_json.as_bytes().to_vec())
        .send()
        .await?;

    println!("Profile created successfully!");
    Ok(())
}


async fn get_last_read(client: &Client, keypair: &Keypair) -> Result<i64> {
    let url = format!("pubky://{}/pub/pubky.app/last_read", keypair.public_key());
    let response = client.get(&url).send().await?;
    let body = response.bytes().await?;
    let last_read: LastRead = serde_json::from_slice(&body)?;
    Ok(last_read.timestamp)
}

async fn update_last_read(client: &Client, keypair: &Keypair, timestamp: i64) -> Result<()> {
    let last_read = LastRead { timestamp };
    let last_read_json = serde_json::to_string(&last_read)?;
    let url = format!("pubky://{}/pub/pubky.app/last_read", keypair.public_key());
    
    client.put(&url)
        .body(last_read_json.as_bytes().to_vec())
        .send()
        .await?;

    println!("Updated last_read to timestamp: {}", timestamp);
    Ok(())
}

async fn check_notifications(client: &Client, keypair: &Keypair) -> Result<()> {
    let last_read = get_last_read(client, keypair).await?;
    println!("Current last_read: {}", last_read);

    let http_client = reqwest::Client::new();
    let nexus_url = env::var("NEXT_PUBLIC_NEXUS").map_err(|_| anyhow::anyhow!("NEXT_PUBLIC_NEXUS not found in .env"))?;
    let url = format!("{}/v0/user/{}/notifications?skip=0&limit=30&since={}", nexus_url, keypair.public_key(), last_read);
    
    println!("Checking notifications from: {}", url);

    let response = http_client.get(&url).send().await?;
    let status = response.status();
    println!("Response status: {}", status);
    
    let response_text = response.text().await?;
    println!("Raw response: {}", response_text);
    
    if response_text.is_empty() {
        println!("Warning: Received empty response");
        return Ok(());
    }

    let notifications: Vec<Notification> = serde_json::from_str(&response_text)?;
    println!("Successfully parsed {} notifications", notifications.len());

    let mut last_timestamp = last_read;

    for notification in notifications {
        if notification.timestamp > last_read {
            match notification.body.notification_type.as_str() {
                "mention" => {
                    if let (Some(mentioned_by), Some(post_uri)) = (notification.body.mentioned_by, notification.body.post_uri) {
                        println!("Received mention from: {}", mentioned_by);
                        
                        let post_content = get_post_content(client, &post_uri).await?;
                        println!("Original post content: {}", post_content);

                        let response = generate_response(&post_content).await?;
                        println!("Generated response: {}", response);

                        let timestamp = Timestamp::now();
                        let post = PubkyAppPost {
                            content: response,
                            kind: PubkyAppPostKind::Short,
                            parent: Some(post_uri),
                            embed: None,
                            attachments: None,
                        };

                        let post_json = serde_json::to_string(&post)?;
                        let url = format!("pubky://{}/pub/pubky.app/posts/{}", keypair.public_key(), timestamp);
                        
                        client.put(&url)
                            .body(post_json.as_bytes().to_vec())
                            .send()
                            .await?;

                        println!("Replied to mention successfully!");
                    }
                }
                "follow" => {
                    if let Some(followed_by) = notification.body.followed_by {
                        println!("Received follow from: {}", followed_by);
                    }
                }
                _ => println!("Received unknown notification type: {}", notification.body.notification_type),
            }

            if notification.timestamp > last_timestamp {
                last_timestamp = notification.timestamp;
            }
        }
    }

    if last_timestamp > last_read {
        update_last_read(client, keypair, last_timestamp + 1).await?;
        
        let new_last_read = get_last_read(client, keypair).await?;
        println!("Verifying last_read update - New value: {}", new_last_read);
        if new_last_read != last_timestamp + 1 {
            println!("WARNING: last_read was not updated correctly!");
            println!("Expected: {}, Got: {}", last_timestamp + 1, new_last_read);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let (client, keypair) = setup_client().await?;
    create_profile(&client, &keypair).await?;
    // create_hello_world_post(&client, &keypair).await?;

    println!("Starting notification polling...");
    loop {
        if let Err(e) = check_notifications(&client, &keypair).await {
            println!("Error checking notifications: {}", e);
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
} 