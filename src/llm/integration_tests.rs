#[cfg(test)]
mod integration_tests {
    use super::super::*;

    #[test]
    #[ignore]
    fn test_simple_chat_completion() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let llm = LLM::new(api_token, "glm-5".to_string());

        let response = llm.send_message("Hello, how are you?".to_string());

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(!resp.choices.is_empty());
        assert!(resp.choices[0].message.content.is_some());
    }

    #[test]
    #[ignore]
    fn test_chat_completion_with_temperature() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let mut llm = LLM::new(api_token, "glm-5".to_string());
        llm.set_default_temperature(0.5);

        let response = llm.send_message("Write a short poem about AI.".to_string());

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(!resp.choices.is_empty());
    }

    #[test]
    #[ignore]
    fn test_chat_completion_with_max_tokens() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let mut llm = LLM::new(api_token, "glm-5".to_string());
        llm.set_default_max_tokens(100);

        let response = llm.send_message("Tell me about the history of computing.".to_string());

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(!resp.choices.is_empty());
    }

    #[test]
    #[ignore]
    fn test_conversation() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let llm = LLM::new(api_token, "glm-5".to_string());
        let mut conv = Conversation::new(llm, "You are a helpful assistant.".to_string());

        let response1 = conv.send("What is 2 + 2?".to_string());
        assert!(response1.is_ok());
        assert!(response1.unwrap().contains("4"));

        let response2 = conv.send("What did I just ask?".to_string());
        assert!(response2.is_ok());
        assert!(response2.unwrap().contains("2 + 2"));
    }

    #[test]
    #[ignore]
    fn test_parallel_requests() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let llm = LLM::new(api_token, "glm-5".to_string());

        let messages = vec![
            vec!["What is 1 + 1?".to_string()],
            vec!["What is 2 + 2?".to_string()],
            vec!["What is 3 + 3?".to_string()],
        ];

        let responses = llm.send_messages_parallel(messages);

        assert_eq!(responses.len(), 3);
        for response in responses {
            assert!(response.is_ok());
            assert!(!response.unwrap().choices.is_empty());
        }
    }

    #[test]
    #[ignore]
    fn test_chat_completion_with_tools() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let llm = LLM::new(api_token, "glm-5".to_string());

        let tool = Tool::function(Function {
            name: "get_weather".to_string(),
            description: "Get weather information for the specified city.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "City Name"
                    }
                },
                "required": ["city"]
            }),
        });

        let messages = vec![Message::new(
            Role::User,
            "What's the weather in Beijing today?",
        )];

        let mut request = ChatCompletionRequest::new(llm.model.clone(), messages);
        request.tools = Some(vec![tool]);
        request.tool_choice = Some("auto".to_string());

        let response = llm.send_request(request);

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(!resp.choices.is_empty());
    }

    #[test]
    #[ignore]
    fn test_vision_model_with_image() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let llm = LLM::new(api_token, "glm-4.6v".to_string());

        let content = vec![
            MultimodalContentItem::ImageUrl {
                image_url: MediaUrl {
                    url: "https://cdn.bigmodel.cn/static/logo/register.png".to_string(),
                },
            },
            MultimodalContentItem::Text {
                text: "What do you see in this image?".to_string(),
            },
        ];

        let messages = vec![Message::new(
            Role::User,
            MessageContent::Multimodal(content),
        )];

        let response = llm.send_chat_completion(messages);

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(!resp.choices.is_empty());
    }

    #[test]
    #[ignore]
    fn test_json_response_format() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let llm = LLM::new(api_token, "glm-5".to_string());

        let messages = vec![Message::new(
            Role::User,
            "Return a JSON object with fields 'name' and 'age' for a person named Alice who is 30 years old.".to_string(),
        )];

        let mut request = ChatCompletionRequest::new(llm.model.clone(), messages);
        request.response_format = Some(ResponseFormat {
            format_type: ResponseFormatType::JsonObject,
        });

        let response = llm.send_request(request);

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(!resp.choices.is_empty());

        let content = resp.choices[0].message.content.as_ref().unwrap();
        let json: serde_json::Value = serde_json::from_str(content).unwrap();
        assert_eq!(json["name"], "Alice");
        assert_eq!(json["age"], 30);
    }

    #[test]
    #[ignore]
    fn test_thinking_mode() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let llm = LLM::new(api_token, "glm-5".to_string());

        let messages = vec![Message::new(
            Role::User,
            "Solve this puzzle: If I have 5 apples and I eat 2, how many do I have left? Explain your reasoning.".to_string(),
        )];

        let mut request = ChatCompletionRequest::new(llm.model.clone(), messages);
        request.thinking = Some(Thinking {
            thinking_type: ThinkingType::Enabled,
            clear_thinking: Some(true),
        });

        let response = llm.send_request(request);

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(!resp.choices.is_empty());

        let message = &resp.choices[0].message;
        assert!(message.reasoning_content.is_some() || message.content.is_some());
    }

    #[test]
    #[ignore]
    fn test_usage_stats() {
        let api_token =
            std::env::var("ZAI_API_TOKEN").expect("ZAI_API_TOKEN environment variable not set");

        let llm = LLM::new(api_token, "glm-5".to_string());

        let response = llm.send_message("Hello!".to_string());

        assert!(response.is_ok());
        let resp = response.unwrap();

        assert!(resp.usage.prompt_tokens > 0);
        assert!(resp.usage.completion_tokens > 0);
        assert!(resp.usage.total_tokens > 0);
        assert_eq!(
            resp.usage.total_tokens,
            resp.usage.prompt_tokens + resp.usage.completion_tokens
        );
    }

    #[test]
    #[ignore]
    fn test_error_handling() {
        let llm = LLM::new("invalid_token".to_string(), "glm-5".to_string());

        let response = llm.send_message("Hello!".to_string());

        assert!(response.is_err());
        match response {
            Err(LLMError::ApiError(err)) => {
                assert!(err.code > 0);
                assert!(!err.message.is_empty());
            }
            _ => panic!("Expected ApiError"),
        }
    }
}
