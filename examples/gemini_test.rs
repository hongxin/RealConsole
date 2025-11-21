//! Gemini Integration Test
//!
//! Manual test to verify Gemini API integration works
//!
//! Usage:
//!   export GEMINI_API_KEY="your-key-here"
//!   cargo run --example gemini_test

use realconsole::llm::{GeminiClient, LlmClient, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Gemini Integration Test\n");

    // Load .env file
    dotenvy::dotenv().ok();

    // Check API key
    let api_key = std::env::var("GEMINI_API_KEY").expect(
        "❌ GEMINI_API_KEY not set.\n\
         Please add it to .env file:\n\
         GEMINI_API_KEY=your-key-here\n\
         \n\
         Get your key from: https://aistudio.google.com/app/apikey",
    );

    println!("✓ API key found (length: {})", api_key.len());

    // Create client with gemini-3-pro-preview (latest, most advanced model)
    let client = GeminiClient::new(
        api_key,
        "gemini-3-pro-preview",
        "https://generativelanguage.googleapis.com",
    )?;
    println!("✓ Client created (model: {})\n", client.model());

    // Test 1: Simple chat
    println!("Test 1: Simple chat");
    println!("-------------------");
    let messages = vec![Message::user("Say 'Hello from Gemini!' if you can read this")];

    match client.chat(messages).await {
        Ok(response) => {
            println!("✓ Response:");
            println!("{}\n", response);
        }
        Err(e) => {
            eprintln!("❌ Error: {}\n", e);
            return Err(e.into());
        }
    }

    // Test 2: With system message
    println!("Test 2: With system message");
    println!("----------------------------");
    let messages = vec![
        Message::system("You are a helpful assistant. Respond very concisely."),
        Message::user("What is 2+2?"),
    ];

    match client.chat(messages).await {
        Ok(response) => {
            println!("✓ Response:");
            println!("{}\n", response);
        }
        Err(e) => {
            eprintln!("❌ Error: {}\n", e);
            return Err(e.into());
        }
    }

    // Test 3: Multi-turn conversation
    println!("Test 3: Multi-turn conversation");
    println!("--------------------------------");
    let messages = vec![
        Message::user("My name is Alice"),
        Message::assistant("Nice to meet you, Alice!"),
        Message::user("What's my name?"),
    ];

    match client.chat(messages).await {
        Ok(response) => {
            println!("✓ Response:");
            println!("{}\n", response);

            if response.to_lowercase().contains("alice") {
                println!("✅ Context maintained!");
            } else {
                println!("⚠️  Warning: May not maintain context");
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}\n", e);
            return Err(e.into());
        }
    }

    // Stats
    println!("\nStats");
    println!("-----");
    let stats = client.stats();
    println!("Calls: {}", stats.total_calls());
    println!("Success: {}", stats.total_success());
    println!("Errors: {}", stats.total_errors());
    println!("Retries: {}", stats.total_retries());

    println!("\n✅ All integration tests passed!");

    Ok(())
}
