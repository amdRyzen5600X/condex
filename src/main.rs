use std::error::Error;
use std::fs;

use condex::{
    Event, EventReader,
    llm::{AsyncConversation, AsyncResult, Conversation, LLM},
    widgets::{Body, BodyState, InputBox, InputBoxState, Title},
};
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Offset, Position},
    widgets::ScrollDirection,
};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let content = "".to_owned();
    let row_count = content.lines().count();
    let mut body_state = BodyState::new(row_count);
    let mut input_state = InputBoxState::new();

    let llm = LLM::new("-".to_owned(), "glm-4.7-flash".to_owned());

    let system_prompt = fs::read_to_string("prompts/system.txt")
        .unwrap_or_else(|_| "You are a helpful assistant.".to_string());
    let conv = Conversation::new(llm, system_prompt);
    let async_conv = AsyncConversation::new(conv);

    let (event_reader, event_sender) = EventReader::with_channel(Duration::from_millis(250));

    loop {
        terminal.draw(|frame| render(frame, &mut body_state, &mut input_state))?;

        match event_reader.read_with_tick() {
            Ok(Event::Input(input_event)) => {
                handle_input(
                    input_event,
                    &mut input_state,
                    &mut body_state,
                    &async_conv,
                    &event_sender,
                );
            }
            Ok(Event::ApiResponse(result)) => {
                handle_api_response(result, &async_conv, &mut body_state);
            }
            Ok(Event::Tick) => {
                // Periodic updates, animations, etc.
            }
            Ok(Event::Terminate) => {
                break Ok(());
            }
            Err(_) => {
                break Ok(());
            }
        }
    }
}

fn handle_input(
    input_event: crossterm::event::Event,
    input_state: &mut InputBoxState,
    body_state: &mut BodyState,
    async_conv: &AsyncConversation,
    event_sender: &mpsc::Sender<Event>,
) {
    match input_event {
        crossterm::event::Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Down => {
                        body_state.scrollbar_state.scroll(ScrollDirection::Forward);
                    }
                    KeyCode::Up => {
                        body_state.scrollbar_state.scroll(ScrollDirection::Backward);
                    }
                    KeyCode::Backspace => {
                        input_state.clear_last_char();
                    }
                    KeyCode::Enter if key.modifiers == KeyModifiers::ALT => {
                        input_state.insert_char('\n');
                    }
                    KeyCode::Enter => {
                        if !input_state.text.is_empty() && !body_state.is_loading {
                            let message = input_state.text.clone();
                            input_state.text.clear();
                            input_state.cursor_offset = Offset::new(0, 0);
                            body_state.is_loading = true;

                            let receiver = async_conv.send_async(message);
                            let sender = event_sender.clone();
                            thread::spawn(move || {
                                if let Ok(result) = receiver.recv() {
                                    let _ = sender.send(Event::ApiResponse(result));
                                }
                            });
                        }
                    }
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        let _ = event_sender.send(Event::Terminate);
                    }
                    key_code => {
                        if let Some(chr) = key_code.as_char() {
                            input_state.insert_char(chr);
                        }
                    }
                }
            }
        }
        crossterm::event::Event::Mouse(mouse) => {
            handle_mouse(mouse, body_state);
        }
        crossterm::event::Event::Resize(_, _) => {
            // Handle resize if needed
        }
        _ => {}
    }
}

fn handle_api_response(
    result: AsyncResult,
    async_conv: &AsyncConversation,
    body_state: &mut BodyState,
) {
    match async_conv.apply_async_result(result) {
        Ok(_) => {
            body_state.is_loading = false;
            body_state.history = async_conv.messages();
        }
        Err(e) => {
            body_state.is_loading = false;
            eprintln!("Error: {}", e);
        }
    }
}

fn handle_mouse(mouse: MouseEvent, state: &mut BodyState) {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            state.scrollbar_state.scroll(ScrollDirection::Forward);
        }
        MouseEventKind::ScrollUp => {
            state.scrollbar_state.scroll(ScrollDirection::Backward);
        }
        _ => {}
    }
}

fn render(frame: &mut Frame, body_state: &mut BodyState, input_state: &mut InputBoxState) {
    let title = Title::new("Condex".to_owned(), frame.area().width, 3);
    let body = Body::new();
    let input_box = InputBox::new();

    let layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![
            Constraint::Max(5),
            Constraint::Min(30),
            Constraint::Max(10),
        ])
        .split(frame.area());

    frame.render_widget(title, layout[0]);
    frame.render_stateful_widget(body, layout[1], body_state);
    frame.render_stateful_widget(input_box, layout[2], input_state);
    let const_offset = Offset::new((layout[2].x + 1) as i32, (layout[2].y + 1) as i32);
    let position = Position::ORIGIN
        .offset(const_offset)
        .offset(input_state.cursor_offset);
    frame.set_cursor_position(position);
}
