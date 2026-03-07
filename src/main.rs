use std::{error::Error, fs, time::Duration};

use condex::widgets::{Body, BodyState, InputBox, InputBoxState, Title};
use crossterm::event::{
    Event, KeyCode, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind, poll,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::ScrollDirection,
};

fn main() -> Result<(), Box<dyn Error>> {
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let content = fs::read_to_string("./test.txt").unwrap();
    let row_count = content.lines().count();
    let mut body_state = BodyState::new(row_count);
    let mut input_state = InputBoxState::new();

    loop {
        terminal.draw(|frame| render(frame, &content, &mut body_state, &mut input_state))?;
        if poll(Duration::from_millis(16))? {
            if let Event::Key(key) = crossterm::event::read()? {
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
                            // TODO: send the text
                        }
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            break Ok(());
                        }
                        key_code => {
                            if let Some(chr) = key_code.as_char() {
                                input_state.insert_char(chr);
                            }
                        }
                    }
                }
            } else if let Event::Mouse(mouse) = crossterm::event::read()? {
                handle_mouse(mouse, &mut body_state);
            }
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

fn render(
    frame: &mut Frame,
    content: &str,
    body_state: &mut BodyState,
    input_state: &mut InputBoxState,
) {
    let title = Title::new("Condex".to_owned(), frame.area().width, 3);
    let body = Body::new(content.to_string());
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
}
