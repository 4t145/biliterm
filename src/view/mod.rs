// use std::{sync::Arc, pin::Pin};

// use tui::{widgets::{Widget, Block, List, Borders, Paragraph}, style::Style, text::{Spans, Span}, layout::{Layout, Rect}};

// use crate::page::Page;

// pub struct PageView<'v>(pub &'v Page);

// impl<'v> Widget for PageView<'v> {
//     fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
//         match self.0 {
//             Page::Home(qrcode) => {
//                 let block = Block::default().borders(Borders::ALL);
//                 let inner = block.inner(area);
//                 block.render(area, buf);
//                 match qrcode {
//                     Some(code) => {
//                         let p = Paragraph::new(code.as_str());
//                         p.render(inner, buf);
//                     },
//                     None => {
//                         buf.set_string(inner.x, inner.y, "Hello Biliterm", Style::default());
//                         buf.set_string(inner.x, inner.y+1, "No QrCode", Style::default());
//                     },
//                 }
//             }
//             Page::LiveRoom(liveroom) => {
//                 let block = Block::default().borders(Borders::ALL);
//                 let inner = block.inner(area);
//                 let width = inner.width;
//                 let top = inner.top();
//                 let mut line = inner.bottom();
//                 let left_bound = inner.left()+1;
//                 for danmaku in liveroom.danmaku_buffer.iter().rev() {
//                     match danmaku {
//                         bilive_danmaku::event::Event::Danmaku { junk_flag:_, message, user, fans_medal:_ } => {
//                             if line == top {
//                                 break;
//                             }
//                             let mut user_name = Span::from(user.uname.as_str());
//                             user_name.style = crate::style::INV;
//                             let message = Span::from(message.to_string());
//                             let msg = Spans::from(vec![user_name, message]);
//                             let p = Paragraph::new(msg);
//                             p.render(Rect::new(left_bound, line,  width, 1), buf);
//                             line -= 1;
//                         },
//                         _ => {}
//                     }
//                 }
//                 block.render(area, buf);
//             },
//         }
//     }
// }