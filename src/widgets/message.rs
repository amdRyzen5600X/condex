use ratatui::widgets::Widget;

use crate::llm::Role;

pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self { content, role }
    }
}

impl Widget for Message {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        match self.role {
            Role::User => {
                let text = ratatui::text::Text::from(self.content).right_aligned();
                text.render(area, buf);
            }
            Role::Assistant => {
                let text = ratatui::text::Text::from(self.content).left_aligned();
                text.render(area, buf);
            }
            _ => {}
        }
    }
}
