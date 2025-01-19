use std::process::{Command, Stdio};
use std::io::{Write, BufReader, BufRead};
use std::time::Duration;
use std::thread::sleep;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum DeviceType {
    Controller,
    Device
}

#[derive(PartialEq, Eq, Clone)]
pub struct Device {
    pub devicetype: DeviceType,
    addr: String,
    pub name: String,
    changelog: Vec<String>
}

fn read() {
    let mut output = Command::new("bluetoothctl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().expect("Err.");

    let interface = match output.stdin.as_mut() {
        Some(interface) => interface,
        None => return
    };

    let _ = interface.write_all(b"scan on\n");
    sleep(Duration::from_secs(5));

    let stdout = match output.stdout.as_mut() {
        Some(stdout) => stdout,
        None => return
    };

    for line in BufReader::new(stdout).lines() {
        
    }

    let _ = interface.write_all(b"exit\n");

    println!("Waiting for process to exit.");
    let _ = output.wait();
    println!("Process exiting.");
}
