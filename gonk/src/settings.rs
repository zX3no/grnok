use crate::{widgets::*, Frame, Widget};
use crossterm::event::MouseEvent;
use gonk_player::{default_device, devices, Device};
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders},
};

pub struct Settings {
    pub devices: Vec<Device>,
    pub index: Option<usize>,
    pub current_device: String,
}

impl Settings {
    pub fn new(wanted_device: &str) -> Self {
        let default = unsafe { default_device() };

        let devices = devices();
        let current_device = if devices.iter().any(|device| device.name == wanted_device) {
            wanted_device.to_string()
        } else {
            default.name.to_string()
        };

        Self {
            index: if devices.is_empty() { None } else { Some(0) },
            devices,
            current_device,
        }
    }

    pub fn selected(&self) -> Option<&str> {
        if let Some(index) = self.index {
            if let Some(device) = self.devices.get(index) {
                return Some(&device.name);
            }
        }
        None
    }
}

impl Widget for Settings {
    fn up(&mut self) {
        if self.devices.is_empty() {
            return;
        }

        match self.index {
            Some(0) => self.index = Some(self.devices.len() - 1),
            Some(n) => self.index = Some(n - 1),
            None => (),
        }
    }

    fn down(&mut self) {
        if self.devices.is_empty() {
            return;
        }

        match self.index {
            Some(n) if n + 1 < self.devices.len() => self.index = Some(n + 1),
            Some(_) => self.index = Some(0),
            None => (),
        }
    }

    fn left(&mut self) {}

    fn right(&mut self) {}

    fn draw(&mut self, f: &mut Frame, area: Rect, _: Option<MouseEvent>) {
        draw(self, area, f);
    }
}

pub fn draw(settings: &mut Settings, area: Rect, f: &mut Frame) {
    let devices: Vec<&str> = settings
        .devices
        .iter()
        .map(|device| device.name.as_str())
        .collect();

    //TODO: I liked the old item menu bold selections.
    //It doesn't work on most terminals though :(
    let mut items: Vec<ListItem> = devices
        .iter()
        .map(|name| {
            if *name == settings.current_device {
                ListItem::new(Spans::from(vec![
                    Span::styled(
                        ">> ",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::DIM | Modifier::BOLD),
                    ),
                    Span::styled(*name, Style::default().add_modifier(Modifier::BOLD)),
                ]))
            } else {
                ListItem::new(format!("   {name}"))
            }
        })
        .collect();

    if let Some(index) = settings.index {
        let item = items[index]
            .clone()
            .style(Style::default().fg(Color::Black).bg(Color::White));
        items[index] = item;
    }

    let list = List::new(&items)
        .block(
            Block::default()
                .title("─Output Device")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default());

    let mut state = ListState::default();
    state.select(settings.index);

    f.render_stateful_widget(list, area, &mut state);
}
