
use bilive_danmaku::event::Event as LvEvent;
use tokio::broadcast;
use futures::{future::FutureExt, select, StreamExt};
use crossterm::{
    event::{self as xtevent, DisableMouseCapture, EnableMouseCapture, Event as XtEvent, KeyCode, EventStream as XtStream},
};


pub enum Event {
    CrossTerm(XtEvent),
    LiveEvent {
        roomid: number,
        evt: LvEvent
    },
    LoggerEvnet
}

pub struct Bus {
    xtstream: XtStream,
    lvstream: Vec<(u64, broadcast::Receiver<Event>)>
}

impl Bus {
    fn new() {
        let xtstream: XtStream = xtevent::EventStream::new();
        Self {
            xtstream,
            lvstream: Vec::new()
        }
    }

    async fn next(&mut self) {
        tokio::select! {
            xt = self.xtevent.next() => {
                xt
            },
            
        }
    }
}
