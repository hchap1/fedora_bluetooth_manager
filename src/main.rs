mod application;
mod bluetooth;
mod utility;

use ratatui::DefaultTerminal;
use application::event_loop;
use bluetooth::{bluetooth, Device};
use utility::*;

use std::thread::spawn;

fn main() {
    let devices: AMV<Device> = sync(vec![]);
    let user_input: AMV<char> = sync(vec![]);
    let running: AM<bool> = sync(true);
    let execute: AM<bool> = sync(false);
    let terminal: AM<DefaultTerminal> = sync(ratatui::init());

    let terminal_clone = terminal.clone();
    let devices_clone = devices.clone();
    let user_input_clone = user_input.clone();
    let running_clone = running.clone();
    let execute_clone = execute.clone();

    let event_handle = spawn(move || event_loop(terminal.clone(), devices.clone(), user_input.clone(), running.clone(), execute.clone()));
    let _bluetooth_handle = spawn(move || bluetooth(terminal_clone, devices_clone, user_input_clone, running_clone, execute_clone));
    let _ = event_handle.join();

    ratatui::restore();
}
