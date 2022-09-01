use std::collections::VecDeque;
use std::sync::Arc;


use bilibili_client::{Client, ClientConfig, ClientError};
use bilibili_client::logger::{Logger, Log, LogLevel};
use tokio::sync::{RwLock, watch};

use tui::buffer;
use tui::text::{Spans, Span};
use crate::style::*;

pub struct WebApiLogger<'l> {
    level: LogLevel,
    size: usize,
    pub buffer: watch::Sender<VecDeque<Spans<'l>>>,
    pub qrcode: watch::Sender<Option<String>>
}

impl<'l> WebApiLogger<'l> {
    pub fn new(size: usize) -> Self {
        let (buffer,_) = watch::channel(VecDeque::with_capacity(size));
        let (qrcode,_) = watch::channel(None);

        Self {
            level: LogLevel::Warn,
            size,
            buffer,
            qrcode,
        }
    }

    pub fn subscribe_qrcode(&self) -> watch::Receiver<Option<String>> {
        self.qrcode.subscribe()
    }

    pub fn subscribe_buffer(&self) -> watch::Receiver<VecDeque<Spans<'l>>> {
        self.buffer.subscribe()
    }
}

impl<'l> Logger for WebApiLogger<'l> {
    type Log = String;

    fn log(&mut self, log: Log<Self::Log>) {
        let mut level;
        match log.level {
            LogLevel::Critical => {
                level = Span::from("CRITICAL");
                level.style = CRITICAL;
            },
            LogLevel::Error => {
                level = Span::from("ERROR");
                level.style = ERROR;
            },
            LogLevel::Warn => {
                level = Span::from("WARN");
                level.style = WARN;
            },
            LogLevel::Info => {
                level = Span::from("INFO");
                level.style = INFO;
            },
            LogLevel::Debug => {
                level = Span::from("DEBUG");
                level.style = DEBUG;
            },
        }

        let msg = Span::from(log.content);
        let log = Spans(vec![level,msg]);
        let size = self.size;
        self.buffer.send_modify(move |b| {
            if b.len() == size {
                b.pop_front();
            } 
            b.push_back(log);
        });
    }

    fn qrcode<'b>(&mut self, bytes: impl Into<&'b [u8]>) {
        use qrcode::{QrCode, render::unicode::Dense1x2};
        let new_code = QrCode::new(bytes.into()).unwrap().render::<Dense1x2>().build();
        self.qrcode.send_modify(move |code|{
            code.replace(new_code);
        })
    }

    fn level(&self) -> &bilibili_client::logger::LogLevel {
        &self.level
    }

    fn level_mut(&mut self) -> &mut bilibili_client::logger::LogLevel {
        &mut self.level
    }

}


pub struct WebApiService<'l> {
    pub bilibili: Arc<RwLock<Client<WebApiLogger<'l>>>>,
    pub watcher: WebApiWatcher<'l>
}

pub struct WebApiWatcher<'l> {
    pub buffer: watch::Receiver<VecDeque<Spans<'l>>>,
    pub qrcode: watch::Receiver<Option<String>>
}

use crate::error::Error;
impl WebApiService<'static> {
    pub fn new() -> Result<Self, Error> {
        use std::path::Path;
        let config = ClientConfig {
            logger: WebApiLogger::new(32),
            cookie_file: Some(Path::new("./webapi.cookie")),
        };
        let client = Client::new(config).map_err(Error::WebApiClientFail)?;
        let watcher = WebApiWatcher {
            buffer: client.logger.subscribe_buffer(),
            qrcode: client.logger.subscribe_qrcode(),
        };
        
        let bilibili = Arc::new(RwLock::new(client));
        Ok(Self {
            bilibili,
            watcher
        })
    }

    // pub async fn watch(&self) -> WebApiWatcher<'static> {
    //     let bilibili = self.bilibili.read().await;
    //     let logger = &bilibili.logger;
    //     let buffer = logger.subscribe_buffer();
    //     let qrcode = logger.subscribe_qrcode();
    //     WebApiWatcher {
    //         buffer, qrcode
    //     }
    // }   

    pub fn spawn_login(&self) {
        let bilibili = self.bilibili.clone();
        tokio::spawn(async move {
            let mut bilibili = bilibili.write().await;
            bilibili.login().await.unwrap();
        });
    }
}