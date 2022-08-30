use std::{io::{self}, thread, time::Duration};
use modal::GlobalState;
use tui::{
    backend::{CrosstermBackend, Backend},
    widgets::{Block, Borders, Tabs, Paragraph, ListItem, List},
    layout::{Layout, Constraint, Direction, Rect},
    Terminal, Frame, text::{Spans, Span, Text}, style::{Style, Color, Modifier}, symbols
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use view::PageView;

mod view;
mod style;
mod modal;

enum InputMode {
    Normal,
    Editing,
}
/// App holds the state of the application
struct App {
    state: GlobalState<'static>,
}

impl App {
    fn new() -> Self{Self{
        state: GlobalState::default(),
    }}

    fn tabs(&self) -> Tabs {
        let titles = self.state.pages.iter().map(|p|Spans::from(p.0.clone())).collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().title("Tabs").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("/");
        tabs
    }

    fn page(&self) -> (&str, PageView) {
        match self.state.current_page {
            Some(idx) => {
                let (title, page) = &self.state.pages[idx];
                (title, page.view())
            }
            None => {
                ("HOME", modal::Page::Home.view())
            },
        }
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
    let (_title, page) = app.page();
    f.render_widget(page, chunks[1]);
}


fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // terminal.draw(window)?;
    let app = App::new();
    terminal.draw(|f|render(f,&app))?;
    thread::sleep(Duration::from_millis(10000));

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;


    Ok(())
}