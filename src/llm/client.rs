use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rayon::prelude::*;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{HeaderMap, HeaderValue};

use crate::llm::types::{
    ChatCompletionRequest, ChatCompletionResponse, ErrorResponse, Message, Role,
};

const DEFAULT_API_ENDPOINT: &str = "https://api.z.ai/api/paas/v4/chat/completions";
const HEADER_ACCEPT_LANGUAGE: &str = "Accept-Language";
const HEADER_ACCEPT_LANGUAGE_VALUE: &str = "en-US,en";
const HEADER_CONTENT_TYPE: &str = "Content-Type";
const HEADER_CONTENT_TYPE_VALUE: &str = "application/json";
const HEADER_AUTHORIZATION: &str = "Authorization";
const BEARER_PREFIX: &str = "Bearer ";

#[derive(Clone, Debug)]
pub struct LLM {
    pub endpoint: String,
    pub api_token: String,
    pub model: String,
    pub client: Client,
    pub default_temperature: Option<f32>,
    pub default_max_tokens: Option<i32>,
}

#[derive(Debug, Clone)]
pub enum LLMError {
    RequestError(String),
    ResponseError(String),
    DeserializationError(String),
    ApiError(ErrorResponse),
    ThreadError(String),
}

impl std::fmt::Display for LLMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMError::RequestError(msg) => write!(f, "Request error: {}", msg),
            LLMError::ResponseError(msg) => write!(f, "Response error: {}", msg),
            LLMError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            LLMError::ApiError(err) => write!(f, "API error: {} - {}", err.code, err.message),
            LLMError::ThreadError(msg) => write!(f, "Thread error: {}", msg),
        }
    }
}

impl std::error::Error for LLMError {}

impl LLM {
    pub fn new(api_token: String, model: String) -> Self {
        Self::with_endpoint(DEFAULT_API_ENDPOINT.to_string(), api_token, model)
    }

    pub fn with_endpoint(endpoint: String, api_token: String, model: String) -> Self {
        let headers = Self::build_headers(&api_token);
        let client = Self::build_client(headers);

        Self {
            endpoint,
            api_token,
            model,
            client,
            default_temperature: None,
            default_max_tokens: None,
        }
    }

    pub fn set_default_temperature(&mut self, temperature: f32) {
        self.default_temperature = Some(temperature);
    }

    pub fn set_default_max_tokens(&mut self, max_tokens: i32) {
        self.default_max_tokens = Some(max_tokens);
    }

    fn build_headers(api_token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            HEADER_ACCEPT_LANGUAGE,
            HeaderValue::from_static(HEADER_ACCEPT_LANGUAGE_VALUE),
        );
        headers.insert(
            HEADER_CONTENT_TYPE,
            HeaderValue::from_static(HEADER_CONTENT_TYPE_VALUE),
        );
        headers.insert(
            HEADER_AUTHORIZATION,
            HeaderValue::from_str(&format!("{}{}", BEARER_PREFIX, api_token))
                .expect("Invalid API token format"),
        );
        headers
    }

    fn build_client(headers: HeaderMap) -> Client {
        ClientBuilder::new()
            .default_headers(headers)
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client")
    }

    pub fn send_message(&self, message: String) -> Result<ChatCompletionResponse, LLMError> {
        let messages = vec![Message::new(Role::User, message)];
        self.send_chat_completion(messages)
    }

    pub fn send_chat_completion(
        &self,
        messages: Vec<Message>,
    ) -> Result<ChatCompletionResponse, LLMError> {
        let mut request = ChatCompletionRequest::new(self.model.clone(), messages);
        request.temperature = self.default_temperature;
        request.max_tokens = self.default_max_tokens;
        self.send_request(request)
    }

    pub fn send_request(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, LLMError> {
        self.send_request_with_timeout(request, Duration::from_secs(120))
    }

    pub fn send_request_with_timeout(
        &self,
        request: ChatCompletionRequest,
        timeout: Duration,
    ) -> Result<ChatCompletionResponse, LLMError> {
        let endpoint = self.endpoint.clone();
        let client = self.client.clone();
        let api_token = self.api_token.clone();

        let (sender, receiver) = mpsc::channel();

        rayon::spawn(move || {
            let result = Self::execute_request(&endpoint, client, &api_token, request);
            let _ = sender.send(result);
        });

        receiver
            .recv_timeout(timeout)
            .map_err(|e| LLMError::ThreadError(format!("Thread communication failed: {}", e)))?
    }

    fn execute_request(
        endpoint: &str,
        client: Client,
        api_token: &str,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, LLMError> {
        let body = serde_json::to_vec(&request).map_err(|e| {
            LLMError::DeserializationError(format!("Failed to serialize request: {}", e))
        })?;

        let response = client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_token))
            .header("Accept-Language", HEADER_ACCEPT_LANGUAGE_VALUE)
            .header("Content-Type", HEADER_CONTENT_TYPE_VALUE)
            .body(body)
            .send()
            .map_err(|e| LLMError::RequestError(format!("Failed to send request: {}", e)))?;

        let status = response.status();

        if !status.is_success() {
            let response_text = response
                .text()
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(LLMError::ApiError(error_response));
            }

            return Err(LLMError::ResponseError(format!(
                "Request failed with status {}: {}",
                status, response_text
            )));
        }

        let response_text = response
            .text()
            .map_err(|e| LLMError::ResponseError(format!("Failed to read response: {}", e)))?;

        let chat_response: ChatCompletionResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                LLMError::DeserializationError(format!(
                    "Failed to deserialize response: {} - Response: {}",
                    e, response_text
                ))
            })?;

        Ok(chat_response)
    }

    pub fn send_messages_parallel(
        &self,
        messages_list: Vec<Vec<String>>,
    ) -> Vec<Result<ChatCompletionResponse, LLMError>> {
        messages_list
            .into_par_iter()
            .map(|messages| {
                let api_messages: Vec<Message> = messages
                    .into_iter()
                    .map(|m| Message::new(Role::User, m))
                    .collect();
                self.send_chat_completion(api_messages)
            })
            .collect()
    }

    pub fn create_conversation(&self, system_prompt: String) -> Conversation {
        Conversation::new(self.clone(), system_prompt)
    }

    pub fn messages_from_strings(&self, messages: Vec<(Role, String)>) -> Vec<Message> {
        messages
            .into_iter()
            .map(|(role, content)| Message::new(role, content))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Conversation {
    llm: LLM,
    messages: Vec<Message>,
}

impl Conversation {
    pub fn new(llm: LLM, system_prompt: String) -> Self {
        let messages = vec![Message::new(Role::System, system_prompt)];
        Self { llm, messages }
    }

    pub fn add_message(&mut self, role: Role, content: String) {
        self.messages.push(Message::new(role, content));
    }

    pub fn send(&mut self, message: String) -> Result<String, LLMError> {
        self.add_message(Role::User, message);
        let response = self.llm.send_chat_completion(self.messages.clone())?;

        if let Some(choice) = response.choices.first() {
            let content = choice.message.content.clone().unwrap_or_default();
            self.add_message(Role::Assistant, content.clone());
            Ok(content)
        } else {
            Err(LLMError::ResponseError(
                "No choices in response".to_string(),
            ))
        }
    }

    pub fn send_with_full_response(
        &mut self,
        message: String,
    ) -> Result<ChatCompletionResponse, LLMError> {
        self.add_message(Role::User, message);
        let response = self.llm.send_chat_completion(self.messages.clone())?;

        if let Some(choice) = response.choices.first() {
            if let Some(content) = &choice.message.content {
                self.add_message(Role::Assistant, content.clone());
            }
        }

        Ok(response)
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn clear(&mut self) {
        if let Some(system_msg) = self.messages.first() {
            self.messages = vec![system_msg.clone()];
        } else {
            self.messages.clear();
        }
    }

    pub fn send_async(&self, message: String) -> mpsc::Receiver<AsyncResult> {
        let (sender, receiver) = mpsc::channel();

        let llm = self.llm.clone();
        let messages = self.messages.clone();

        rayon::spawn(move || {
            let mut local_messages = messages.clone();
            local_messages.push(Message::new(Role::User, message.clone()));

            let result = llm.send_chat_completion(local_messages.clone());

            let async_result = match result {
                Ok(response) => {
                    if let Some(choice) = response.choices.first() {
                        let content = choice.message.content.clone().unwrap_or_default();
                        let assistant_msg = Message::new(Role::Assistant, content.clone());
                        AsyncResult::Success {
                            user_message: Message::new(Role::User, message),
                            assistant_message: assistant_msg,
                            full_response: response,
                        }
                    } else {
                        AsyncResult::Error(LLMError::ResponseError(
                            "No choices in response".to_string(),
                        ))
                    }
                }
                Err(e) => AsyncResult::Error(e),
            };

            let _ = sender.send(async_result);
        });

        receiver
    }
}

#[derive(Debug, Clone)]
pub enum AsyncResult {
    Success {
        user_message: Message,
        assistant_message: Message,
        full_response: ChatCompletionResponse,
    },
    Error(LLMError),
}

#[derive(Clone)]
pub struct AsyncConversation {
    inner: Arc<Mutex<Conversation>>,
}

impl AsyncConversation {
    pub fn new(conv: Conversation) -> Self {
        Self {
            inner: Arc::new(Mutex::new(conv)),
        }
    }

    pub fn send_async(&self, message: String) -> mpsc::Receiver<AsyncResult> {
        let (sender, receiver) = mpsc::channel();
        let conv_clone = Arc::clone(&self.inner);

        rayon::spawn(move || {
            let conv = conv_clone.lock().unwrap();
            let llm = conv.llm.clone();
            let messages = conv.messages.clone();
            drop(conv);

            let mut local_messages = messages.clone();
            local_messages.push(Message::new(Role::User, message.clone()));

            let result = llm.send_chat_completion(local_messages.clone());

            let async_result = match result {
                Ok(response) => {
                    if let Some(choice) = response.choices.first() {
                        let content = choice.message.content.clone().unwrap_or_default();
                        let assistant_msg = Message::new(Role::Assistant, content.clone());
                        AsyncResult::Success {
                            user_message: Message::new(Role::User, message),
                            assistant_message: assistant_msg,
                            full_response: response,
                        }
                    } else {
                        AsyncResult::Error(LLMError::ResponseError(
                            "No choices in response".to_string(),
                        ))
                    }
                }
                Err(e) => AsyncResult::Error(e),
            };

            let _ = sender.send(async_result);
        });

        receiver
    }

    pub fn add_message(&self, role: Role, content: String) {
        let mut conv = self.inner.lock().unwrap();
        conv.add_message(role, content);
    }

    pub fn messages(&self) -> Vec<Message> {
        let conv = self.inner.lock().unwrap();
        conv.messages().to_vec()
    }

    pub fn apply_async_result(&self, result: AsyncResult) -> Result<String, LLMError> {
        match result {
            AsyncResult::Success {
                user_message,
                assistant_message,
                full_response: _,
            } => {
                let mut conv = self.inner.lock().unwrap();
                conv.messages.push(user_message);
                conv.messages.push(assistant_message.clone());
                let content = match &assistant_message.content {
                    crate::llm::types::MessageContent::Text(s) => s.clone(),
                    _ => String::new(),
                };
                Ok(content)
            }
            AsyncResult::Error(e) => Err(e),
        }
    }

    pub fn is_response_ready(receiver: &mpsc::Receiver<AsyncResult>) -> bool {
        receiver.try_recv().is_ok()
    }

    pub fn try_recv_result(receiver: &mpsc::Receiver<AsyncResult>) -> Option<AsyncResult> {
        receiver.try_recv().ok()
    }

    pub fn clear(&self) {
        let mut conv = self.inner.lock().unwrap();
        conv.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_creation() {
        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        assert_eq!(llm.model, "glm-5");
        assert_eq!(llm.api_token, "test_token");
    }

    #[test]
    fn test_llm_with_endpoint() {
        let llm = LLM::with_endpoint(
            "https://custom.endpoint.com".to_string(),
            "test_token".to_string(),
            "glm-4.7".to_string(),
        );
        assert_eq!(llm.endpoint, "https://custom.endpoint.com");
    }

    #[test]
    fn test_default_values() {
        let mut llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        assert!(llm.default_temperature.is_none());
        assert!(llm.default_max_tokens.is_none());

        llm.set_default_temperature(0.7);
        assert_eq!(llm.default_temperature, Some(0.7));

        llm.set_default_max_tokens(1024);
        assert_eq!(llm.default_max_tokens, Some(1024));
    }

    #[test]
    fn test_conversation_creation() {
        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let conv = Conversation::new(llm, "You are a helpful assistant".to_string());
        assert_eq!(conv.messages.len(), 1);
        assert_eq!(conv.messages[0].role, Role::System);
    }

    #[test]
    fn test_conversation_add_message() {
        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let mut conv = Conversation::new(llm, "You are a helpful assistant".to_string());
        conv.add_message(Role::User, "Hello".to_string());
        assert_eq!(conv.messages.len(), 2);
        assert_eq!(conv.messages[1].role, Role::User);
    }

    #[test]
    fn test_conversation_clear() {
        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let mut conv = Conversation::new(llm, "You are a helpful assistant".to_string());
        conv.add_message(Role::User, "Hello".to_string());
        conv.clear();
        assert_eq!(conv.messages.len(), 1);
        assert_eq!(conv.messages[0].role, Role::System);
    }

    #[test]
    fn test_async_conversation_creation() {
        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let conv = Conversation::new(llm, "You are a helpful assistant".to_string());
        let async_conv = AsyncConversation::new(conv);
        assert_eq!(async_conv.messages().len(), 1);
    }

    #[test]
    fn test_async_conversation_add_message() {
        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let conv = Conversation::new(llm, "You are a helpful assistant".to_string());
        let async_conv = AsyncConversation::new(conv);
        async_conv.add_message(Role::User, "Hello".to_string());
        assert_eq!(async_conv.messages().len(), 2);
    }

    #[test]
    fn test_async_conversation_clear() {
        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let conv = Conversation::new(llm, "You are a helpful assistant".to_string());
        let async_conv = AsyncConversation::new(conv);
        async_conv.add_message(Role::User, "Hello".to_string());
        async_conv.clear();
        assert_eq!(async_conv.messages().len(), 1);
    }

    #[test]
    fn test_async_conversation_clonable() {
        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let conv = Conversation::new(llm, "You are a helpful assistant".to_string());
        let async_conv1 = AsyncConversation::new(conv);
        let async_conv2 = async_conv1.clone();

        async_conv1.add_message(Role::User, "Hello".to_string());

        assert_eq!(async_conv2.messages().len(), 2);
    }

    #[test]
    fn test_async_result_success() {
        let user_msg = Message::new(Role::User, "Hello".to_string());
        let assistant_msg = Message::new(Role::Assistant, "Hi there!".to_string());

        let result = AsyncResult::Success {
            user_message: user_msg.clone(),
            assistant_message: assistant_msg.clone(),
            full_response: ChatCompletionResponse {
                id: "test".to_string(),
                request_id: None,
                created: 0,
                model: "test".to_string(),
                choices: vec![],
                usage: crate::llm::types::Usage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                    prompt_tokens_details: None,
                },
                web_search: None,
            },
        };

        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let conv = Conversation::new(llm, "System".to_string());
        let async_conv = AsyncConversation::new(conv);

        match async_conv.apply_async_result(result) {
            Ok(content) => {
                assert_eq!(content, "Hi there!");
                let messages = async_conv.messages();
                assert_eq!(messages.len(), 3);
                assert_eq!(messages[1].role, Role::User);
                assert_eq!(messages[2].role, Role::Assistant);
            }
            Err(_) => panic!("Expected success"),
        }
    }

    #[test]
    fn test_async_result_error() {
        let error = LLMError::RequestError("Test error".to_string());
        let result = AsyncResult::Error(error.clone());

        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let conv = Conversation::new(llm, "System".to_string());
        let async_conv = AsyncConversation::new(conv);

        match async_conv.apply_async_result(result) {
            Err(e) => assert_eq!(e.to_string(), "Request error: Test error"),
            Ok(_) => panic!("Expected error"),
        }
    }
}
