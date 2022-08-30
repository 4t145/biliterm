use std::{collections::HashMap, pin::Pin};
use std::sync::{RwLock, RwLockReadGuard};
use tui::widgets::Widget;

use crate::view::PageView;



pub enum Page {
    Home
}

impl Page {
    pub fn view(&self) -> PageView {
        PageView(self)
    }
}


pub struct GlobalState<'p> {
    pub pages: Vec<(String, RwLockReadGuard<'p, Page>)>,
    pub current_page: Option<usize>,
}

impl<'p> Default for GlobalState<'p> {
    fn default() -> Self {
        Self {
            current_page: None,
            pages: Vec::new(),
        }
    }
}