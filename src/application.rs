use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Stylize, Style, Modifier},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, List},
    DefaultTerminal
};

use crate::bluetooth::{Device, DeviceType};
use crate::utility::{AM, AMV};
const SELECTED_STYLE: Style = Style::new().bg(ratatui::style::Color::Red).add_modifier(Modifier::BOLD);

fn render_frame(frame: &mut Frame, devices: &Vec<Device>, user_input: &Vec<char>) {
    let layout = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(60),
            Constraint::Percentage(20),
            Constraint::Percentage(20)
        ]).split(frame.area());

    let mut paired: Vec<usize> = vec![];
    let mut available: Vec<usize> = vec![];
    let mut connected: Vec<usize> = vec![];

    for (idx, device) in devices.iter().enumerate() {
        if device.connected { connected.push(idx); }
        else if device.paired { paired.push(idx); }
        else { available.push(idx); }
    }

    let title = Line::from(" [RUST BLUETOOTH MANAGER] ").bold().centered().blue();
    let available_title = Line::from(" [AVAILABLE DEVICES] ").bold().centered().white();
    let paired_title = Line::from(" [PAIRED DEVICES] ").bold().centered().yellow();
    let connected_title = Line::from(" [CONNECTED DEVICES] ").bold().centered().green();
    let input = Line::from(format!(" > {} ", user_input.iter().collect::<String>())).green();

    let available_list = available.iter().filter(|idx| devices[**idx].devicetype == DeviceType::Device).map(|idx| Line::from(vec![
            format!("DEVICE {}: ", idx).white(),
            devices[*idx].name.to_string().white()
        ])).collect::<Vec<Line>>();

    let paired_list = Text::from(
        paired.iter().filter(|idx| devices[**idx].devicetype == DeviceType::Device).map(|idx| Line::from(vec![
            format!("DEVICE {}: ", idx).yellow(),
            devices[*idx].name.to_string().white()
        ])).collect::<Vec<Line>>()
    );

    let connected_list = Text::from(
        connected.iter().filter(|idx| devices[**idx].devicetype == DeviceType::Device).map(|idx| Line::from(vec![
            format!("DEVICE {}: ", idx).green(),
            devices[*idx].name.to_string().white()
        ])).collect::<Vec<Line>>()
    );
    let l = List::new(available_list).block(Block::bordered().title(available_title).border_set(border::THICK)).highlight_style(SELECTED_STYLE);

    frame.render_widget(l, layout[0]);
    frame.render_widget(Paragraph::new(paired_list).centered().block(Block::bordered().title(paired_title).title_bottom(input).border_set(border::THICK)), layout[1]);
    frame.render_widget(Paragraph::new(connected_list).centered().block(Block::bordered().title(connected_title).border_set(border::THICK)), layout[2]);
}

pub fn update_screen(terminal: AM<DefaultTerminal>, devices: AMV<Device>, user_input: AMV<char>) -> Result<(), std::io::Error> {
    let mut terminal = terminal.lock().unwrap();
    let devices = devices.lock().unwrap();
    let user_input = user_input.lock().unwrap();
    match terminal.draw(|frame| render_frame(frame, &devices, &user_input)) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

fn event_callback(ke: KeyEvent, user_input: &mut Vec<char>, running: &mut bool, execute: &mut bool) {
    match ke.code {
        KeyCode::Esc => { *running = false; }
        KeyCode::Backspace => { user_input.pop(); }
        KeyCode::Char(c) => { user_input.push(c); }
        KeyCode::Enter => { *execute = true; }
        _ => {}
    }
}

pub fn event_loop(terminal: AM<DefaultTerminal>, devices: AMV<Device>, user_input: AMV<char>, running: AM<bool>, execute: AM<bool>) {
    loop {
        let e = match event::read() {
            Ok(e) => e,
            Err(_) => {
                let mut running = running.lock().unwrap();
                *running = false;
                return;
            }
        };

        match e {
            Event::Key(ke) if ke.kind == KeyEventKind::Press => {
                let mut user_input = user_input.lock().unwrap();
                let mut running = running.lock().unwrap();
                let mut execute = execute.lock().unwrap();
                event_callback(ke, &mut user_input, &mut running, &mut execute);
                if !*running { break; }
            }
            _ => {}
        };
        {
            let _ = update_screen(terminal.clone(), devices.clone(), user_input.clone());
        }
    }
}
