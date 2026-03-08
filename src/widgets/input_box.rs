use ratatui::{
    layout::{Offset, Rect},
    text::Text,
    widgets::{Block, StatefulWidget, Widget},
};

pub struct InputBox {}

pub struct InputBoxState {
    pub text: String, // TODO: chage String to Rope data structure
    pub mode: Mode,
    pub max_width: usize,
    pub cursor_offset: Offset,
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
            text: "".to_owned(),
            mode: Mode::Normal,
            max_width: 0,
            cursor_offset: Offset::ZERO,
        }
    }

    pub fn clear_last_char(&mut self) {
        if let Some(chr) = self.text.pop() {
            match chr {
                '\n' => {
                    self.cursor_offset.y -= 1;
                    self.cursor_offset.x =
                        self.text.lines().last().map(str::len).unwrap_or(0) as i32;
                }
                _ => {
                    self.cursor_offset.x -= 1;
                }
            }
        }
    }

    pub fn insert_char(&mut self, chr: char) {
        if chr.eq(&'\n') {
            self.cursor_offset.y += 1;
            self.cursor_offset.x = 0;
        } else {
            self.cursor_offset.x += 1;
        }
        self.text.push(chr);
        if let Some(last_line) = self.text.lines().last() {
            if last_line.len() >= self.max_width && !self.text.ends_with('\n') {
                self.text.push('\n');
                self.cursor_offset.y += 1;
                self.cursor_offset.x = 0;
            }
        }
    }
}
