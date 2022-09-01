use tui::{
    backend::{CrosstermBackend, Backend},
    widgets::{Widget, Block, Borders, Tabs, Paragraph, ListItem, List},
    layout::{Layout, Constraint, Direction, Rect},
    Terminal, Frame, text::{Spans, Span, Text}, style::{Style, Color, Modifier}, symbols
};

pub const CRITICAL: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::Magenta),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty()
};

pub const ERROR: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::Red),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty()
};

pub const WARN: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::Yellow),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty()
};

pub const INFO: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::Blue),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty()
};

pub const DEBUG: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty()
};

pub const INV: Style = Style {
    fg: Some(Color::Black),
    bg: Some(Color::White),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty()
};