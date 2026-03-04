use std::error::Error;

use condex::widgets::Title;
use ratatui::{DefaultTerminal, Frame};

fn main() -> Result<(), Box<dyn Error>> {
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let title = Title::new("Condex".to_owned(), frame.area().width, 3);
    frame.render_widget(title, frame.area());
}
