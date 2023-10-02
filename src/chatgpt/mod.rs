use chatgpt::{prelude::*, types::CompletionResponse};

pub async fn testing_chatgpt() {
    let client = ChatGPT::new("").unwrap();
    // Sending a message and getting the completion
    let response: CompletionResponse = client
        .send_message("Describe in five words the Rust programming language.")
        .await
        .unwrap();

    println!("Response: {}", response.message().content);
}
