use std::{io::{self}};
use bilibili_client::danmaku;
use futures::{StreamExt};
use page::GlobalState;
use service::webapi::{WebApiService};

use tui::{
    backend::{CrosstermBackend, Backend},
    widgets::{Block, Borders, Tabs, Paragraph},
    layout::{Layout, Constraint, Direction, Rect, Alignment},
    Terminal, Frame, text::{Spans,Text}, style::{Style, Color}
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as XtEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::{error::Error, page::{liveroom::LiveRoomPageService, PageService, Psh, login::{LoginPageService}}};


mod view;
#[allow(dead_code)]
mod style;
mod page;
// mod event;
mod error;
mod service;

/// App holds the state of the application
pub struct App {
    state: GlobalState,
    #[allow(dead_code)]
    webapi_service: WebApiService,
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
                let (_, handle) = &self.state.pages[idx];
                handle.render(f, area);
            }
            None => {
                // let qrcode = self.webapi_service.watcher.qrcode.borrow().clone();
                f.render_widget(Paragraph::new("WELCOME").alignment(Alignment::Center).style(style::INFO), area);
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
            let display = format!("[{action}]:{buffer}");
            app.render_single_line_input(f, chunks[2], display);
        },
        page::InputState::Normal => {
            app.render_message(f, chunks[2]);
        },
    }
}

#[derive(Debug)]
pub enum Evnet {
    Tick,
    Xt(XtEvent),
    Error
}

pub struct EventCable {
    ticker: tokio::time::Interval,
    oubound: tokio::sync::mpsc::UnboundedSender<Evnet>,
}

impl EventCable {
    async fn run(self) {
        let ob = self.oubound.clone();
        // for xterm event
        tokio::spawn(async move {
            let mut reader = event::EventStream::new();
            while let Some(Ok(e)) = reader.next().await {
                ob.send(Evnet::Xt(e)).unwrap_or_default()
            }
        });
        // for ticker
        let ob = self.oubound.clone();
        let mut ticker = self.ticker;
        tokio::spawn(async move {
            loop {
                ticker.tick().await;
                ob.send(Evnet::Tick).unwrap_or_default()
            }
        });
    }
}
// 此处逻辑需要拆分
async fn run<B:Backend>(app: &mut App, terminal: &mut Terminal<B>) -> Result<(), Error> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let cable = EventCable {
        ticker: tokio::time::interval(tokio::time::Duration::from_millis(1000)),
        oubound: tx
    };
    tokio::spawn(cable.run());
    let webapi_service = crate::service::webapi::WebApiService::new()?;
    // let mut rerender_timer = tokio::time::interval(tokio::time::Duration::from_millis(500));
    // let online = { webapi_service.bilibili.is_online() };
    // if !online {
    //     let oauth_key = webapi_service.fetch_qrcode().await?;
    //     loop {
    //         webapi_service.try_login(oauth_key)
    //     }
    // }
    terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
    // Racing
    while let Some(e) = rx.recv().await {
        match e {
            Evnet::Tick => {
                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
            },
            Evnet::Xt(e) => {
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
                            (Char('w'), Press, KeyModifiers::CONTROL) => {
                                app.state.close_page();
                                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                            }
                            (Char('r'), Press, KeyModifiers::CONTROL) => {
                                app.state.input_state = page::InputState::edit_action(Action::CreatLiveRoomPage);
                                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                            }
                            (Char('l'), Press, KeyModifiers::CONTROL) => {
                                let srv = LoginPageService::new(&webapi_service.bilibili);
                                let psh = Psh::LoginPageService(srv.run());
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                app.state.message(format!("psh is running"));
                                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                                app.state.regist_page(format!("登录"), psh);
                                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                            }
                            (Char(',')|Tab, Press, KeyModifiers::CONTROL)|(PageDown, Press, KeyModifiers::NONE) => {
                                app.state.to_next_page();
                                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                            }
                            (Char('.'), Press, KeyModifiers::CONTROL)|(PageUp, Press, KeyModifiers::NONE) => {
                                app.state.to_prev_page();
                                terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                            }
                            (Char(c), Press, KeyModifiers::NONE) => {
                                let next = match &mut app.state.input_state {
                                    page::InputState::EditAction { action:_, display:_, buffer } => {
                                        buffer.push(c);
                                        terminal.draw(|f|render(f, &app)).map_err(Error::Io)?;
                                        None
                                    },
                                    page::InputState::Normal => {
                                        if let Some(Psh::LiveRoomPageService(p)) = app.state.current_page_psh() {
                                            if c == 't' {
                                                let next = page::InputState::edit_action(Action::SendDanmakuToLive(p.watcher.borrow().roomid));
                                                Some(next)
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    },
                                };
                                next.map(|s|app.state.input_state = s);
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
                            (Esc, Press, KeyModifiers::NONE) => {
                                match &mut app.state.input_state {
                                    s@page::InputState::EditAction { .. } => {
                                        *s = page::InputState::Normal;
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
                                            Action::CreatLiveRoomPage => {
                                                match buffer.parse::<u64>() {
                                                    Ok(roomid) => {
                                                        let srv = LiveRoomPageService::new(roomid).await.unwrap();
                                                        app.state.regist_page(format!("直播{roomid}"), Psh::LiveRoomPageService(srv.run()));
                                                    },
                                                    Err(e) => {
                                                        app.state.message(format!("{e}"))
                                                    },
                                                }
                                            },
                                            Action::SendDanmakuToLive(roomid) => {
                                                webapi_service.bilibili.excute(bilibili_client::transaction::send_danmaku_to_live::SendDanmakuToLive {
                                                    roomid: *roomid,
                                                    danmaku: danmaku!(buffer.as_str())
                                                });
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
                    }
                    _ => {

                    }
                    // XtEvent::Paste(_) => todo!(),
                    // XtEvent::FocusGained => todo!(),
                    // XtEvent::FocusLost => todo!(),
                    // XtEvent::Mouse(_) => todo!(),
                    // XtEvent::Resize(_, _) => todo!(),
                }
            },
            Evnet::Error => todo!(),
        }
    }
    Ok(())
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