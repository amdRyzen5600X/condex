use ratatui::{
    layout::Rect,
    text::Text,
    widgets::{
        Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget as _,
    },
};

pub struct Body {
    pub history: String,
}

impl Body {
    pub fn new(content: String) -> Self {
        Self { history: content }
    }
}

pub struct BodyState {
    pub is_modifying: bool,
    pub scrollbar_state: ScrollbarState,
}

impl BodyState {
    pub fn new(rows: usize) -> Self {
        let max_scroll = rows.saturating_sub(1);
        let mut scrollbar_state = ScrollbarState::new(max_scroll).content_length(rows);
        scrollbar_state.last();
        Self {
            is_modifying: false,
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

        let lines: Vec<&str> = self.history.lines().collect();
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

        let text = Text::from_iter(lines.into_iter().rev().skip(start_line).take(lines_to_show).rev());
        text.render(inner, buf);

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
