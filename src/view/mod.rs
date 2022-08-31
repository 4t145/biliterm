use std::{sync::Arc, pin::Pin};

use tui::{widgets::{Widget, Block, List, Borders, }, style::Style};

use crate::page::Page;

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
            Page::LiveRoom(liveroom) => {
                let block = Block::default().borders(Borders::ALL);
                let inner = block.inner(area);
                let mut line = inner.bottom();
                for danmaku in &liveroom.danmaku_buffer {
                    match danmaku {
                        bilive_danmaku::event::Event::Danmaku { junk_flag:_, message, user, fans_medal:_ } => {
                            let msg = format!("{}:{message}",user.uname);
                            buf.set_string(inner.x, line, msg, Style::default());
                            line -= 1;
                            if line == 0 {
                                break;
                            }
                        },
                        _ => {}
                    }
                }
                block.render(area, buf);
            },
        }
    }
}