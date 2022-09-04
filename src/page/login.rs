use std::sync::Arc;

use bilibili_client::{transaction::{login::{Login, LoginState}}, Client};
use tokio::sync::watch;
use tui::{widgets::{Widget, Block, Borders, Paragraph}, layout::{Alignment, Layout, Direction, Constraint}};

use super::{PageService, PageServiceHandle};

// 
// 
// 
// 
// 
// 
// 
// 
#[derive(Debug, Default)]
pub struct LoginPage {
    qrcode: Option<String>,
    lint: String
}

impl<'a> Widget for &'a LoginPage {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Min(10),
                ]
                .as_ref(),
            )
        .split(inner);
        block.render(area, buf);
        let lint = Paragraph::new(self.lint.as_str()).alignment(Alignment::Center);
        lint.render(chunks[0], buf);
        let qrcode = match &self.qrcode {
            Some(code) => {
                Paragraph::new(code.as_str()).alignment(Alignment::Center)
            },
            None => Paragraph::new("No QrCode").alignment(Alignment::Center).style(crate::style::INFO),
        };
        qrcode.render(chunks[1], buf);
    }
}

pub struct LoginPageService {
    pub client: Arc<Client>,
}

impl LoginPageService {
    pub fn new(client: &Arc<Client>) -> Self {
        Self {
            client: client.clone()
        }
    }
}

impl PageService for LoginPageService {
    type Page = LoginPage;
    fn run(self) -> PageServiceHandle<Self::Page> {
        let client_login_task = self.client.excute(Login{});
        let (tx, watcher) = watch::channel(LoginPage::default());
        let task = async move {
            let mut state = client_login_task.state;
            loop {
                state.changed().await.unwrap();
                let state = &*state.borrow();
                match state {
                    LoginState::FetchingQrcode => {
                        tx.send_modify(|p|{p.lint = "请求二维码中".into()})
                    },
                    LoginState::ScaningQrcode(qr) => {
                        tx.send_modify(|p|{p.qrcode = Some(
                            qrcode::QrCode::new(qr.as_bytes()).unwrap().render::<qrcode::render::unicode::Dense1x2>().build()
                        )})
                    },
                    LoginState::QrcodeExpired => {
                        tx.send_modify(|p|{p.lint = "二维码已过期".into()})
                    },
                    LoginState::QrcodeScanFinished => {
                        tx.send_modify(|p|{
                            p.lint = "已扫描，登陆中".into();
                            p.qrcode = None;
                        })
                    },
                    LoginState::UnexpectedCode(c) => {
                        tx.send_modify(|p|{p.lint = format!("意外的状态码: {c}")});
                        break;
                    },
                    LoginState::Success { url:_ } => {
                        tx.send_modify(|p|{
                            p.lint = "登录成功".into();
                            p.qrcode = None;
                        });
                        break;
                    },
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