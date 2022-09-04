use std::{fmt::Display};
use tokio::sync::{watch};
use tokio::task::JoinHandle;
use tui::{widgets::Widget, Frame, backend::Backend, layout::Rect};



// use crate::view::PageView;
pub mod liveroom;
pub mod login;
use self::login::LoginPageService;
use self::liveroom::LiveRoomPageService;

macro_rules! psh {
    ($($page:ident),*) => {
        pub enum Psh {
            $($page(PageServiceHandle<<$page as PageService>::Page>),)*
        }

        impl Psh {
            pub fn render<B:Backend>(&self, f: &mut Frame<B>, area: Rect) {
                match self {
                    $(Self::$page(h) => {
                        // <$page as PageService>::render(&*h.watcher.borrow(), app, f, area)
                        f.render_widget(&*h.watcher.borrow(), area)
                    },)*
                }
            }

            pub fn abort(self) {
                match self {
                    $(Self::$page(h) => {
                        // <$page as PageService>::render(&*h.watcher.borrow(), app, f, area)
                        h.handle.abort()
                    },)*
                }
            }
        }
    };
}

psh!(
    LoginPageService,
    LiveRoomPageService
);

pub struct PageServiceHandle<P> {
    pub watcher: watch::Receiver<P>,
    pub handle: JoinHandle<()>
}

pub trait PageService:Sized 
where for <'a> &'a Self::Page: Widget
{
    type Page;
    fn run(self) -> PageServiceHandle<Self::Page>;
}

pub struct GlobalState {
    pub pages: Vec<(String, Psh)>,
    pub current_page: Option<usize>,
    pub messages: Vec<String>,
    pub input_state: InputState,
}

#[derive(Clone)]
pub enum Action {
    CreatLiveRoomPage,
    SendDanmakuToLive(u64)
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::CreatLiveRoomPage => {
                f.write_str("创建房间")
            },
            Action::SendDanmakuToLive(_) => {
                f.write_str("发送弹幕")
            },
        }
    }
}

#[derive(Clone)]
pub enum InputState {
    EditAction {
        action: Action,
        display: String,
        buffer: String,
    },
     
    Normal,
}

impl InputState {
    pub fn edit_action(action: Action) -> Self {
        Self::EditAction {
            action,
            display: String::new(),
            buffer: String::new()
        }
    }
}


impl Default for InputState {
    fn default() -> Self {
        Self::Normal
    }
}

impl GlobalState {
    pub fn current_page_psh<'a>(&'a self) -> Option<&'a Psh> {
        self.current_page.map(|idx|{
            &self.pages[idx].1
        })
    }
    pub fn regist_page(&mut self, title: String, psh: Psh) {
        self.pages.push((title, psh));
        self.to_last_page();
    }

    pub fn close_page(&mut self) {
        if let Some(idx) = self.current_page {
            let (_, psh) = self.pages.remove(idx);
            psh.abort();
            self.current_page = match self.pages.len() {
                0 => None,
                len if len==idx => Some(0),
                _ => Some(idx)
            }
        }
    }

    pub fn message(&mut self, s:impl Into<String>) {
        self.messages.push(s.into())
    }

    pub fn to_last_page(&mut self) {
        if !self.pages.is_empty() {
            self.current_page.replace(self.pages.len()-1);
        }
    }

    // pub fn to_first_page(&mut self) {
    //     if !self.pages.is_empty() {
    //         self.current_page.replace(0);
    //     }
    // }

    pub fn to_prev_page(&mut self) {
        let len = self.pages.len();
        if self.pages.len() != 0 {
            match self.current_page {
                Some(idx) => {
                    if idx == 0 {
                        self.current_page.replace(len-1);
                    } else {
                        self.current_page.replace(idx-1);
                    }
                }
                None => {
                    self.current_page.replace(len-1);
                }
            }
        }
    }

    pub fn to_next_page(&mut self) {
        let len = self.pages.len();
        if self.pages.len() != 0 {
            match self.current_page {
                Some(idx) => {
                    if idx == len-1 {
                        self.current_page.replace(0);
                    } else {
                        self.current_page.replace(idx+1);
                    }
                }
                None => {
                    self.current_page.replace(0);
                }
            }
        }
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            current_page: None,
            pages: Vec::new(),
            messages: Vec::new(),
            input_state: InputState::default()
        }
    }
}

// pub struct GlobalService {

// }