use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal
};
use std::thread::{spawn, JoinHandle};
use crate::bluetooth::{Device, DeviceType};
use crate::utility::{AM, AMV, sync};

fn render_frame(area: Rect, buffer: &mut Buffer, devices: &Vec<Device>, user_input: &Vec<char>) {
    let title = Line::from(" [RUST BLUETOOTH MANAGER] ").bold().centered().blue();
    let input = Line::from(format!(" > {} ", user_input.iter().collect::<String>())).green();
    let device_list = Text::from(
        devices.iter().filter(|device| device.devicetype == DeviceType::Device).map(|device| Line::from(vec![
            "DEVICE: ".green(),
            device.name.to_string().white()
        ])).collect::<Vec<Line>>()
    );
    let block = Block::bordered().title(title).title_bottom(input).border_set(border::THICK);
    Paragraph::new(device_list).centered().block(block).render(area, buffer);
}

fn update_screen(terminal: AM<DefaultTerminal>, devices: AMV<Device>, user_input: AMV<char>) -> Result<(), std::io::Error> {
    let mut terminal = terminal.lock().unwrap();
    let devices = devices.lock().unwrap();
    let user_input = user_input.lock().unwrap();
    match terminal.draw(|frame| render_frame(frame.area(), frame.buffer_mut(), &devices, &user_input)) {
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
