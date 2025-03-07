use std::process::{Command, Stdio, ChildStdout};
use std::io::{Write, BufReader, BufRead};
use std::thread::{sleep, spawn};
use std::time::Duration;
use ratatui::DefaultTerminal;
use std::mem::replace;
use regex::Regex;

use crate::utility::*;
use crate::application::update_screen;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum DeviceType {
    _Controller,
    Device
}

#[derive(PartialEq, Eq, Clone)]
pub struct Device {
    pub devicetype: DeviceType,
    addr: String,
    pub name: String,
    changelog: Vec<String>,
    pub paired: bool,
    pub connected: bool
}

fn read(terminal: AM<DefaultTerminal>, devices: AMV<Device>, user_input: AMV<char>, running: AM<bool>, stdout: &mut ChildStdout) {
    let r = Regex::new("^..-..-..-..-..-..$").unwrap();
    for line in BufReader::new(stdout).lines() {
        {
            let running = running.lock().unwrap();
            if !*running {
                return;
            }
        }

        let mut components = match line {
            Ok(line) => line.split(' ').map(|x| x.to_string()).collect::<Vec<String>>(),
            Err(_) => {
                let mut running = running.lock().unwrap();
                *running = false;
                return;
            }
        };
        
        if components.len() > 5 {
            while components.len() > 5 {
                let last = components.pop().unwrap();
                let idx = components.len() - 1;
                components[idx] += &format!(" {last}");
            }
        }

        match components.get(2) {
            Some(t) => match t.as_str() {
                "Device" if components.len() >= 5 && components[1] == "\u{1b}[0m[\u{1b}[0;92mNEW\u{1b}[0m]" => {
                    let mut devices = devices.lock().unwrap();
                    let device = Device {
                        devicetype: DeviceType::Device,
                        addr: components[3].clone(),
                        name: components[4].clone(),
                        changelog: vec![],
                        paired: false,
                        connected: false
                    };

                    let mut add: bool = true;

                    if r.is_match(&device.name) {
                        add = false;
                    }

                    for d in devices.iter() {
                        if d.addr == device.addr {
                            add = false;
                            break;
                        }
                    }

                    if add { devices.push(device); }
                }
                _ => {}
            }
            None => continue
        };

        {
            let _ = update_screen(terminal.clone(), devices.clone(), user_input.clone());
        }
    }
}

fn exec_bctl(args: Vec<String>) -> Result<Vec<String>, ()> {
    let output = Command::new("bluetoothctl").args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).output().expect("Failed to execute command.");
    
    match output.status.success() {
        true => Ok(output.stdout.lines().into_iter().map(|x| x.unwrap().to_string()).collect()),
        false => Err(())
    }
}

pub fn bluetooth(terminal: AM<DefaultTerminal>, devices: AMV<Device>, user_input: AMV<char>, running: AM<bool>, execute: AM<bool>) {
    

    let _ = exec_bctl(vec!["power off".into()]);
    let _ = exec_bctl(vec!["power on".into()]);
    let _ = exec_bctl(vec!["agent on".into()]);
    let _ = exec_bctl(vec!["default-agent".into()]);

    let device_output = Command::new("bluetoothctl")
        .arg("devices")
        .arg("Paired")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Err when reading devices.");

    if device_output.status.success() {
        let stdout = String::from_utf8_lossy(&device_output.stdout);
        for line in stdout.lines() {
            let components = line.split(' ').map(|x| x.to_string()).collect::<Vec<String>>();
            if components.len() < 3 { continue; }
            {
                let mut devices = devices.lock().unwrap();       
                devices.push(
                    Device {
                        devicetype: DeviceType::Device,
                        addr: components[1].clone(),
                        changelog: vec![],
                        name: components.iter().skip(2).map(|x| x.clone()).collect::<Vec<String>>().join(" "),
                        paired: true,
                        connected: false
                    }
                )
            }
        }
    }

    let mut output = Command::new("bluetoothctl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().expect("Err.");

    let interface = match output.stdin.as_mut() {
        Some(interface) => interface,
        None => return
    };

    let _ = interface.write_all(b"power on\n");
    let _ = interface.write_all(b"pairable on\n");
    let _ = interface.write_all(b"discoverable on\n");
    let _ = interface.write_all(b"scan on\n");

    let mut stdout = match output.stdout.take() {
        Some(stdout) => stdout,
        None => return
    };

    let devices_clone = devices.clone();
    let user_input_clone = user_input.clone();
    let running_clone = running.clone();

    let _read_handle = spawn(move || read(terminal, devices_clone, user_input_clone, running_clone, &mut stdout));

    sleep(Duration::from_millis(500));

    loop {
        sleep(Duration::from_millis(100));

        {
            let running = running.lock().unwrap();
            if !*running {
                break;
            }
        }
        {
            let mut execute = execute.lock().unwrap();
            if *execute {
                *execute = false;
                let command = {
                    let mut user_input = user_input.lock().unwrap();
                    replace(&mut *user_input, vec![]).iter().collect::<String>().split(' ').map(|x| x.to_string()).collect::<Vec<String>>()
                };

                match command.len() {
                    1 => {
                        match command[0].as_str() {
                            "exit" => {
                                let mut running = running.lock().unwrap();
                                *running = false;
                            }
                            &_ => {}
                        }
                    }
                    2 => {
                        match command[0].as_str() {
                            "pair" => {
                                let device = match command[1].parse::<usize>() {
                                    Ok(device) => device,
                                    Err(_) => continue
                                };
                                let mut devices = devices.lock().unwrap();
                                if device >= devices.len() {
                                    continue;
                                }

                                let mac_address = devices[device].addr.clone();
                                let _ = interface.write_all(format!("pair {mac_address}\n").as_bytes());
                                devices[device].paired = true;
                            }
                            "connect" => {
                                let device = match command[1].parse::<usize>() {
                                    Ok(device) => device,
                                    Err(_) => continue
                                };
                                let mut devices = devices.lock().unwrap();
                                if device >= devices.len() {
                                    continue;
                                }

                                let mac_address = devices[device].addr.clone();
                                let _ = interface.write_all(format!("connect {mac_address}\n").as_bytes());
                                devices[device].connected = true;
                            }
                            "remove" => {
                                let device = match command[1].parse::<usize>() {
                                    Ok(device) => device,
                                    Err(_) => continue
                                };
                                let mut devices = devices.lock().unwrap();
                                if device >= devices.len() {
                                    continue;
                                }

                                let mac_address = devices[device].addr.clone();
                                let _ = interface.write_all(format!("remove {mac_address}\n").as_bytes());
                                devices[device].connected = false;
                                devices[device].paired = false;
                                devices.clear();
                                let _ = interface.write_all(b"scan off\n");
                                let _ = interface.write_all(b"agent off\n");
                                let _ = interface.write_all(b"power off\n");
                                let _ = interface.write_all(b"power on\n");
                                let _ = interface.write_all(b"agent on\n");
                                let _ = interface.write_all(b"default-agent\n");
                                let _ = interface.write_all(b"scan on\n");
                            }
                            &_ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let _ = interface.write_all(b"exit\n");

    println!("Waiting for process to exit.");
    let _ = output.wait();
    println!("Process exiting.");
}
