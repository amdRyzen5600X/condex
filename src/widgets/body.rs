use ratatui::{
    layout::{self, Constraint, Rect},
    text::{Line, Span},
    widgets::{
        Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget as _,
    },
};

use crate::llm::{Message, MessageContent};
use crate::widgets;

fn extract_content(content: &MessageContent) -> String {
    match content {
        MessageContent::Text(s) => s.clone(),
        MessageContent::Multimodal(items) => items
            .iter()
            .filter_map(|item| match item {
                crate::llm::MultimodalContentItem::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

pub struct Body {}

impl Body {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct BodyState {
    pub history: Vec<Message>,
    pub is_modifying: bool,
    pub is_loading: bool,
    pub scrollbar_state: ScrollbarState,
}

impl BodyState {
    pub fn new(rows: usize) -> Self {
        let max_scroll = rows.saturating_sub(1);
        let mut scrollbar_state = ScrollbarState::new(max_scroll).content_length(rows);
        scrollbar_state.last();
        Self {
            is_modifying: false,
            is_loading: false,
            history: Vec::new(),
            scrollbar_state,
        }
    }
}

impl StatefulWidget for Body {
    type State = BodyState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let block = Block::bordered();
        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines: Vec<widgets::Message> = state
            .history
            .iter()
            .map(|msg| widgets::Message::new(msg.role, extract_content(&msg.content)))
            .collect();

        if state.is_loading {
            let loading_indicator = Line::from(vec![
                Span::raw(""),
                Span::styled(
                    "●",
                    ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
                ),
                Span::raw(" "),
                Span::styled(
                    "Thinking...",
                    ratatui::style::Style::default().fg(ratatui::style::Color::Gray),
                ),
            ]);
            let loading_text = loading_indicator.to_string();
            lines.push(widgets::Message::new(
                crate::llm::Role::Assistant,
                loading_text,
            ));
        }

        let total_lines = lines.len();
        let viewport_height = inner.height as usize;
        let max_scroll = total_lines.saturating_sub(viewport_height);

        let current_position = state.scrollbar_state.get_position().min(max_scroll);

        state.scrollbar_state = state
            .scrollbar_state
            .position(current_position)
            .viewport_content_length(viewport_height)
            .content_length(total_lines);

        let lines_to_show = viewport_height.min(total_lines);
        let start_line = max_scroll.saturating_sub(current_position);

        lines = lines
            .into_iter()
            .rev()
            .skip(start_line)
            .take(lines_to_show)
            .rev()
            .collect();
        let layout =
            layout::Layout::vertical(lines.iter().map(|line| {
                Constraint::Length(line.content.lines().collect::<Vec<_>>().len() as u16)
            }))
            .flex(layout::Flex::End)
            .split(inner);
        for (area, msg) in layout.iter().zip(lines) {
            msg.render(*area, buf);
        }

        // text.render(inner, buf);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"));
        scrollbar.render(
            Rect::new(inner.right(), inner.top(), 1, inner.height),
            buf,
            &mut state.scrollbar_state,
        );
    }
}
