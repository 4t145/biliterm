use std::{sync::Arc, pin::Pin};

use tui::{widgets::{Widget, Block, Paragraph, Borders, }, style::Style};

use crate::modal::Page;

pub struct PageView<'v>(pub &'v Page);


impl<'v> Widget for PageView<'v> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        match self.0 {
            Page::Home => {
                let block = Block::default().borders(Borders::ALL);
                let inner = block.inner(area);
                buf.set_string(inner.x, inner.y, "Hello Biliterm", Style::default());
                block.render(area, buf);
            }
        }
    }
}




pub struct HomePageView {
    msg: String   
}

impl Widget for HomePageView {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        buf.set_string(0, 0, "Hello Biliterm", Style::default())
    }
}