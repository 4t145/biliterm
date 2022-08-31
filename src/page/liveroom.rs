
use std::{collections::VecDeque, sync::Arc};

use bilibili_client::api::live;
use bilive_danmaku::{
    model::DanmakuMessage,     
};
use tokio::sync::{watch, RwLockReadGuard};

use crate::error::Error::*;

pub struct LiveRoom {
    pub danmaku_buffer: VecDeque<bilive_danmaku::event::Event>
}



impl LiveRoom {
    pub fn new() -> Self {
        Self { danmaku_buffer: VecDeque::new() }
    }

    pub fn push_danmaku(&mut self, danmaku:bilive_danmaku::event::Event, ) {
        self.danmaku_buffer.push_back(danmaku);
        if self.danmaku_buffer.len() > 64 {
            self.danmaku_buffer.pop_front();
        }
    }
}

pub struct LiveRoomPage {
    
}

use bilive_danmaku::{
    RoomService,
    Connected
};

use super::Page;

pub struct LiveRoomService {
    sender: watch::Sender<Page>,
    room_service: RoomService<Connected>
}




impl LiveRoomService {
    pub async fn new(roomid: u64) -> Result<Self, crate::error::Error> {
        let room_service = RoomService::new(roomid).init().await
        .map_err(|_|ConnectLiveRoomFail)?
        .connect().await.map_err(|_|ConnectLiveRoomFail)?;
        let liveroom = LiveRoom::new();
        let (sender, _) = watch::channel(Page::LiveRoom(liveroom));
        Ok(Self {
            room_service,
            sender,
        })
    }

    pub async fn watch(&self) -> watch::Receiver<Page> {
        self.sender.subscribe()
    }

    pub async fn serve(self) {
        let mut reciever = self.room_service.subscribe();
        while let Ok(e) = reciever.recv().await {
            match e {
                danmaku@bilive_danmaku::event::Event::Danmaku {..} => {
                    self.sender.send_if_modified(|p|{
                        match p {
                            Page::LiveRoom(liveroom) => {
                                liveroom.push_danmaku(danmaku);
                                true
                            },
                            _ => false
                        }
                    });
                },
                _ => {}
            }
        }
    }
}
// pub fn run() {
//     let model = LiveRoom::new();
//     let srv = LiveRoomService::new();
// }