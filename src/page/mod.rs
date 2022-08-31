use std::{collections::HashMap, pin::Pin, fmt::Display};
use tokio::sync::{watch, RwLockReadGuard};
use tui::widgets::Widget;

use crate::view::PageView;
pub mod liveroom;


pub enum Page {
    Home,
    LiveRoom(liveroom::LiveRoom)
}

impl Page {
    pub fn view(&self) -> PageView {
        PageView(self)
    }
}


pub struct GlobalState {
    pub pages: Vec<(String, watch::Receiver<Page>)>,
    pub current_page: Option<usize>,
    pub messages: Vec<String>,
    pub input_state: InputState
}

#[derive(Clone)]
pub enum Action {
    CreatLiveRoomPage
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::CreatLiveRoomPage => {
                f.write_str("创建房间")
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
    pub fn regist_page(&mut self, title: String, page: watch::Receiver<Page>) {
        self.pages.push((title, page));
        self.to_last_page();
    }

    pub fn message(&mut self, s:impl Into<String>) {
        self.messages.push(s.into())
    }

    pub fn to_last_page(&mut self) {
        if !self.pages.is_empty() {
            self.current_page.replace(self.pages.len()-1);
        }
    }

    pub fn to_first_page(&mut self) {
        if !self.pages.is_empty() {
            self.current_page.replace(0);
        }
    }

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

pub struct GlobalService {

}