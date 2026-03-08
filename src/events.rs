use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CrosstermEvent};

use crate::llm::AsyncResult;

#[derive(Debug, Clone)]
pub enum Event {
    Input(CrosstermEvent),
    ApiResponse(AsyncResult),
    Tick,
    Terminate,
}

pub struct EventReader {
    rx: mpsc::Receiver<Event>,
    tx: mpsc::Sender<Event>,
    tick_rate: Duration,
}

impl EventReader {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();

        let tx_clone = tx.clone();
        thread::spawn(move || {
            loop {
                if let Ok(crossterm_event) = event::read() {
                    let app_event = Event::Input(crossterm_event);
                    if tx_clone.send(app_event).is_err() {
                        break;
                    }
                }
            }
        });

        Self { rx, tx, tick_rate }
    }

    pub fn with_channel(tick_rate: Duration) -> (Self, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel();

        let tx_clone = tx.clone();
        thread::spawn(move || {
            loop {
                if let Ok(crossterm_event) = event::read() {
                    let app_event = Event::Input(crossterm_event);
                    if tx_clone.send(app_event).is_err() {
                        break;
                    }
                }
            }
        });

        let reader = Self {
            rx,
            tx: tx.clone(),
            tick_rate,
        };

        (reader, tx)
    }

    pub fn send_api_response(&self, result: AsyncResult) {
        let _ = self.tx.send(Event::ApiResponse(result));
    }

    pub fn send_terminate(&self) {
        let _ = self.tx.send(Event::Terminate);
    }

    pub fn read(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn try_read(&self) -> Option<Event> {
        self.rx.try_recv().ok()
    }

    pub fn read_timeout(&self, timeout: Duration) -> Result<Option<Event>, mpsc::RecvError> {
        let start = Instant::now();

        loop {
            match self.rx.recv_timeout(timeout) {
                Ok(event) => return Ok(Some(event)),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if start.elapsed() >= timeout {
                        return Ok(None);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(mpsc::RecvError);
                }
            }
        }
    }

    pub fn read_with_tick(&self) -> Result<Event, mpsc::RecvError> {
        let start = Instant::now();
        let tick_interval = self.tick_rate;

        loop {
            let elapsed = start.elapsed();

            if elapsed >= tick_interval {
                return Ok(Event::Tick);
            }

            let remaining = tick_interval - elapsed;
            match self.rx.recv_timeout(remaining) {
                Ok(event) => return Ok(event),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    return Ok(Event::Tick);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(mpsc::RecvError);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ChatCompletionResponse, Conversation, LLM, Role};

    #[test]
    fn test_event_creation() {
        let crossterm_event = CrosstermEvent::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        let event = Event::Input(crossterm_event);
        assert!(matches!(event, Event::Input(_)));
    }

    #[test]
    fn test_event_reader_creation() {
        let reader = EventReader::new(Duration::from_millis(100));

        assert_eq!(reader.tick_rate, Duration::from_millis(100));
    }

    #[test]
    fn test_send_api_response() {
        let reader = EventReader::new(Duration::from_millis(100));

        let llm = LLM::new("test_token".to_string(), "glm-5".to_string());
        let conv = Conversation::new(llm, "System".to_string());
        let user_msg = crate::llm::types::Message::new(Role::User, "Hello".to_string());
        let assistant_msg = crate::llm::types::Message::new(Role::Assistant, "Hi".to_string());

        let result = AsyncResult::Success {
            user_message: user_msg,
            assistant_message: assistant_msg,
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

        reader.send_api_response(result);

        if let Some(Event::ApiResponse(_)) = reader.try_read() {
            // Success
        } else {
            panic!("Expected ApiResponse event");
        }
    }

    #[test]
    fn test_tick_event() {
        let reader = EventReader::new(Duration::from_millis(50));

        let event = reader.read_with_tick().unwrap();
        assert!(matches!(event, Event::Tick));
    }
}
