use std::{io::{self}, thread, time::Duration, collections::VecDeque};
use futures::{StreamExt, io::Read};
use page::GlobalState;
use page::liveroom::LiveRoomService;
use service::webapi::{WebApiWatcher, WebApiService};
use tokio::sync::watch;
use tui::{
    backend::{CrosstermBackend, Backend},
    widgets::{Block, Borders, Tabs, Paragraph, ListItem, List},
    layout::{Layout, Constraint, Direction, Rect},
    Terminal, Frame, text::{Spans, Span, Text}, style::{Style, Color, Modifier}, symbols
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as XtEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use view::PageView;
use crate::error::Error;
mod view;
mod style;
mod page;
// mod event;
mod error;
mod service;

enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
struct App {
    state: GlobalState,
    webapi_service: WebApiService<'static>,
}

impl App {
    fn new() -> Self{
        let webapi_service = WebApiService::new().unwrap();
        Self{
            state: GlobalState::default(),
            webapi_service
        }
    }

    fn tabs(&self) -> Tabs {
        let titles = self.state.pages.iter().map(|p|Spans::from(p.0.clone())).collect();
        let tabs = Tabs::new(titles)
            .select(self.state.current_page.unwrap_or(0))
            .block(Block::default().title("Tabs").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("/");
        tabs
    }

    fn render_page<B:Backend>(&self, f: &mut Frame<B>, area: Rect) {
        match self.state.current_page {
            Some(idx) => {
                let (_title, page) = &self.state.pages[idx];
                let page = page.borrow();
                let pageview = page.view();
                f.render_widget(pageview, area);
            }
            None => {
                let qrcode = self.webapi_service.watcher.qrcode.borrow().clone();
                f.render_widget(page::Page::Home(qrcode).view(), area);
            },
        }
    }

    fn render_message<B:Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let msg = self.state.messages.last().map(|x|x.as_str()).unwrap_or_default();
        self.render_single_line_input(f, area, msg);
    }

    fn render_single_line_input<'a, B:Backend>(&self, f: &mut Frame<B>, area: Rect, text: impl Into<Text<'a>>) {
        let text = Paragraph::new(text.into());
        f.render_widget(text, area)
    }
}


fn render<B:Backend>(f: &mut Frame<B>, app: &App) {
    let tabs = app.tabs();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    f.render_widget(tabs, chunks[0]);
    app.render_page(f, chunks[1]);
    match &app.state.input_state {
        page::InputState::EditAction { action, display:_, buffer } => {
            let display = format!("[{action}]roomid:{buffer}");
            app.render_single_line_input(f, chunks[2], display);
        },
        page::InputState::Normal => {
            app.render_message(f, chunks[2]);
        },
    }
}


pub enum Evnet {
    Tick,
    Xt(XtEvent),
    Error
}
// 此处逻辑需要拆分
async fn run<B:Backend>(app: &mut App, terminal: &mut Terminal<B>) -> Result<(), Error> {
    let mut reader = event::EventStream::new();
    let webapi_service = crate::service::webapi::WebApiService::new()?;
    // let mut rerender_timer = tokio::time::interval(tokio::time::Duration::from_millis(500));
    webapi_service.spawn_login();
    terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
    // Racing
    loop {
        let event = reader.next().await;
        terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
        match event {
            Some(r) => {
                match r {
                    Ok(e) => {
                        match e {
                            XtEvent::Key(key_evt) => {
                                use KeyCode::*;
                                use event::KeyEventKind::*;
                                use event::KeyModifiers;
                                use page::Action;
                                match (key_evt.code, key_evt.kind, key_evt.modifiers) {
                                    (Char('c'), Press, KeyModifiers::CONTROL) => {
                                        return Ok(())
                                    }
                                    (Char('l'), Press, KeyModifiers::CONTROL) => {
                                        app.state.input_state = page::InputState::edit_action(Action::CreatLiveRoomPage);
                                        terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                                    }
                                    (PageUp|Char(','), Press, KeyModifiers::CONTROL) => {
                                        app.state.to_next_page();
                                        terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                                    }
                                    (PageDown|Char('.'), Press, KeyModifiers::CONTROL) => {
                                        app.state.to_prev_page();
                                        terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                                    }
                                    (Char(c), Press, KeyModifiers::NONE) => {
                                        match &mut app.state.input_state {
                                            page::InputState::EditAction { action:_, display:_, buffer } => {
                                                buffer.push(c);
                                                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                                            },
                                            page::InputState::Normal => {

                                            },
                                        }
                                    }
                                    (Backspace, Press, KeyModifiers::NONE) => {
                                        match &mut app.state.input_state {
                                            page::InputState::EditAction { action:_, display:_, buffer } => {
                                                buffer.pop();
                                                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                                            },
                                            page::InputState::Normal => {

                                            },
                                        }
                                    }
                                    (Enter, Press, KeyModifiers::NONE) => {
                                        let state= &mut app.state.input_state;
                                        match state {
                                            page::InputState::EditAction { action, display:_, buffer } => {
                                                match action {
                                                    page::Action::CreatLiveRoomPage => {
                                                        match buffer.parse::<u64>() {
                                                            Ok(roomid) => {
                                                                let srv = LiveRoomService::new(roomid).await.unwrap();
                                                                let watcher = srv.watch().await;
                                                                tokio::spawn(srv.serve());
                                                                app.state.regist_page(format!("直播{roomid}"), watcher);
                                                            },
                                                            Err(e) => {
                                                                app.state.message(format!("{e}"))
                                                            },
                                                        }
                                                    },
                                                }
                                            },
                                            page::InputState::Normal => {},
                                        }
                                        app.state.input_state = page::InputState::Normal;
                                        terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                                    }
                                    _ => {

                                    }
                                }

                            },
                            XtEvent::Resize(_, _) => {
                                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                            },
                            _ => {

                            }
                            // Event::FocusGained => todo!(),
                            // Event::FocusLost => todo!(),
                            // Event::Mouse(_) => todo!(),
                            // Event::Paste(_) => todo!(),
                        }
                    },
                    Err(_) => {

                    },
                }
            },
            None => {

            },
        }
    }
}


fn main() -> Result<(), Error> {
    // setup terminal
    enable_raw_mode().map_err(Error::Io)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(Error::Io)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(Error::Io)?;
    
    // terminal.draw(window)?;
    let mut app = App::new();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().thread_name("biliterm").build().map_err(Error::Io)?;
    rt.block_on(run(&mut app, &mut terminal)).unwrap();

    // restore terminal
    disable_raw_mode().map_err(Error::Io)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).map_err(Error::Io)?;
    terminal.show_cursor().map_err(Error::Io)?;


    Ok(())
}