use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::llm::types::Message;
use crate::llm::{AsyncConversation, Conversation, Role, LLM};

#[test]
fn test_non_blocking_behavior() {
    let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
    let conv = Conversation::new(llm, "System".to_string());
    let async_conv = AsyncConversation::new(conv);

    let receiver = async_conv.send_async("Hello".to_string());

    println!("Message sent, not blocked!");

    thread::sleep(Duration::from_millis(100));

    println!("Still not blocked, doing other work...");

    match receiver.try_recv() {
        Ok(result) => {
            println!("Response received!");
            match async_conv.apply_async_result(result) {
                Ok(_) => println!("Success!"),
                Err(e) => println!("Error: {}", e),
            }
        }
        Err(mpsc::TryRecvError::Empty) => {
            println!("No response yet (expected for test without API)");
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            println!("Channel disconnected");
        }
    }
}

#[test]
fn test_parallel_conversation_clones() {
    let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
    let conv = Conversation::new(llm, "System".to_string());
    let async_conv1 = AsyncConversation::new(conv);

    let async_conv2 = async_conv1.clone();
    let async_conv3 = async_conv2.clone();

    assert_eq!(async_conv1.messages().len(), 1);
    assert_eq!(async_conv2.messages().len(), 1);
    assert_eq!(async_conv3.messages().len(), 1);

    async_conv1.add_message(Role::User, "Message 1".to_string());
    async_conv2.add_message(Role::User, "Message 2".to_string());
    async_conv3.add_message(Role::User, "Message 3".to_string());

    assert_eq!(async_conv1.messages().len(), 4);
    assert_eq!(async_conv2.messages().len(), 4);
    assert_eq!(async_conv3.messages().len(), 4);
}
