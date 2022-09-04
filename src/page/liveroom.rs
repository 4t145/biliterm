
use std::{collections::VecDeque};

use tokio::sync::{watch};
use tui::{widgets::{Widget, Block, Borders, Paragraph}, text::{Span, Spans}, layout::Rect};


#[derive(Default)]
pub struct LiveRoomPage {
    pub danmaku_buffer: VecDeque<bilive_danmaku::event::Event>
}

impl LiveRoomPage {
    pub fn push_danmaku(&mut self, danmaku:bilive_danmaku::event::Event, ) {
        self.danmaku_buffer.push_back(danmaku);
        if self.danmaku_buffer.len() > 64 {
            self.danmaku_buffer.pop_front();
        }
    }
}


impl<'a> Widget for &'a LiveRoomPage {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);
        let width = inner.width;
        let top = inner.top();
        let mut line = inner.bottom();
        let left_bound = inner.left()+1;
        for danmaku in self.danmaku_buffer.iter().rev() {
            match danmaku {
                bilive_danmaku::event::Event::Danmaku { junk_flag:_, message, user, fans_medal:_ } => {
                    if line == top {
                        break;
                    }
                    let mut user_name = Span::from(user.uname.as_str());
                    user_name.style = crate::style::INV;
                    let message = Span::from(message.to_string());
                    let msg = Spans::from(vec![user_name, message]);
                    let p = Paragraph::new(msg);
                    p.render(Rect::new(left_bound, line,  width, 1), buf);
                    line -= 1;
                },
                _ => {}
            }
        }
        block.render(area, buf);
    }
}


use bilive_danmaku::{
    RoomService,
    Connected
};

use super::{PageService, PageServiceHandle};

pub struct LiveRoomPageService {
    room_service: RoomService<Connected>
}
impl LiveRoomPageService {
    pub async fn new(roomid: u64) -> Result<Self, ()> {
        let service = bilive_danmaku::RoomService::new(roomid).init().await.map_err(|_|())?.connect().await.map_err(|_|())?;
        Ok(Self {
            room_service: service
        })
    }
}
impl PageService for LiveRoomPageService {
    type Page = LiveRoomPage;

    fn run(self) -> PageServiceHandle<Self::Page> {
        let mut reciever = self.room_service.subscribe();
        let live_room_page = LiveRoomPage::default();
        let (tx,watcher) = watch::channel(live_room_page);
        let task = async move {
            while let Ok(e) = reciever.recv().await {
                match e {
                    danmaku@bilive_danmaku::event::Event::Danmaku {..} => {
                        tx.send_modify(|p|{p.push_danmaku(danmaku)});
                    },
                    _ => {}
                }
            }
        };
        let handle = tokio::spawn(task);
        PageServiceHandle {
            watcher,
            handle
        }
    }
}