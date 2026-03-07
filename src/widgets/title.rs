use ratatui::{
    layout::{Constraint, Rect},
    text::Text,
    widgets::{self, Widget},
};

pub struct Title {
    pub title: String,
    pub width: u16,
    pub hight: u16,
}

impl Title {
    pub fn new(title: String, width: u16, hight: u16) -> Self {
        Self {
            title,
            width,
            hight,
        }
    }
}

impl Widget for Title {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rect = Rect::new(0, 0, self.width, self.hight);
        let block = widgets::Block::bordered();
        let area = block.inner(area);
        block.render(rect, buf);
        let text = Text::raw(self.title);
        let area = area.centered_horizontally(Constraint::Length(text.width() as u16));
        text.render(area, buf);
    }
}
