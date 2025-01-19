mod application;
mod bluetooth;
mod utility;

use ratatui::DefaultTerminal;
use application::event_loop;
use bluetooth::Device;
use utility::*;

use std::thread::spawn;

fn main() {
    let devices: AMV<Device> = sync(vec![]);
    let user_input: AMV<char> = sync(vec![]);
    let running: AM<bool> = sync(true);
    let execute: AM<bool> = sync(false);
    let terminal: AM<DefaultTerminal> = sync(ratatui::init());

    let event_handle = spawn(move || event_loop(terminal.clone(), devices.clone(), user_input.clone(), running.clone(), execute.clone()));
    let _ = event_handle.join();

    ratatui::restore();
}
