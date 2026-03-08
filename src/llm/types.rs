use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "tool")]
    Tool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Multimodal(Vec<MultimodalContentItem>),
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        MessageContent::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        MessageContent::Text(s.to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultimodalContentType {
    Text,
    ImageUrl,
    VideoUrl,
    FileUrl,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MultimodalContentItem {
    Text { text: String },
    ImageUrl { image_url: MediaUrl },
    VideoUrl { video_url: MediaUrl },
    FileUrl { file_url: MediaUrl },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaUrl {
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn new(role: Role, content: impl Into<MessageContent>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }

    pub fn with_tool_call_id(mut self, tool_call_id: String) -> Self {
        self.tool_call_id = Some(tool_call_id);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tool {
    Function { function: Function },
    Retrieval { retrieval: Retrieval },
    WebSearch { web_search: WebSearch },
}

impl Tool {
    pub fn function(function: Function) -> Self {
        Tool::Function { function }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Retrieval {
    pub knowledge_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_template: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchEngine {
    SearchProJina,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchRecencyFilter {
    OneDay,
    OneWeek,
    OneMonth,
    OneYear,
    NoLimit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSize {
    Medium,
    High,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultSequence {
    Before,
    After,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebSearch {
    pub search_engine: SearchEngine,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_domain_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_recency_filter: Option<SearchRecencyFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_size: Option<ContentSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_sequence: Option<ResultSequence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_result: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_prompt: Option<String>,
}

impl WebSearch {
    pub fn new(search_engine: SearchEngine) -> Self {
        Self {
            search_engine,
            enable: None,
            search_query: None,
            count: None,
            search_domain_filter: None,
            search_recency_filter: None,
            content_size: None,
            result_sequence: None,
            search_result: None,
            require_search: None,
            search_prompt: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingType {
    Enabled,
    Disabled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thinking {
    #[serde(rename = "type")]
    pub thinking_type: ThinkingType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clear_thinking: Option<bool>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatType {
    Text,
    JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: ResponseFormatType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub do_sample: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<Thinking>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

impl ChatCompletionRequest {
    pub fn new(model: String, messages: Vec<Message>) -> Self {
        Self {
            model,
            messages,
            request_id: None,
            do_sample: None,
            stream: Some(false),
            thinking: None,
            temperature: None,
            top_p: None,
            max_tokens: None,
            tool_stream: None,
            tools: None,
            tool_choice: None,
            stop: None,
            response_format: None,
            user_id: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<Vec<WebSearchResult>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: ChatCompletionMessage,
    #[serde(rename = "finish_reason")]
    pub finish_reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletionMessage {
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolCallFunction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub content: String,
    pub link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_date: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
}
