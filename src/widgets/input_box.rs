use ratatui::{
    layout::Rect,
    text::Text,
    widgets::{Block, StatefulWidget, Widget},
};

pub struct InputBox {}

pub struct InputBoxState {
    pub text: String, // TODO: chage String to Rope data structure
    pub mode: Mode,
    pub max_width: usize,
}

pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl InputBox {
    pub fn new() -> Self {
        Self {}
    }
}

impl StatefulWidget for InputBox {
    type State = InputBoxState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let rect = Rect::new(area.x, area.y, area.width, area.height);
        let block = Block::bordered();
        let area = block.inner(area);
        state.max_width = area.width.saturating_sub(2) as usize;
        block.render(rect, buf);
        let text = Text::raw(&state.text);
        text.render(area, buf);
    }
}

impl InputBoxState {
    pub fn new() -> Self {
        Self {
            text: "█".to_owned(),
            mode: Mode::Normal,
            max_width: 0,
        }
    }

    pub fn clear_last_char(&mut self) {
        if let Some(c) = self.text.pop() {
            self.text.pop();
            self.text.push(c);
        }
    }

    pub fn insert_char(&mut self, chr: char) {
        if let Some(c) = self.text.pop() {
            self.text.push(chr);
            if let Some(last_line) = self.text.lines().last() {
                if last_line.len() >= self.max_width && !self.text.ends_with('\n') {
                    self.text.push('\n');
                }
            }
            self.text.push(c);
        }
    }
}
