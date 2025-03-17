mod form;
mod scale;
mod spreadsheet;

use std::{io, str, sync::mpsc, thread, time::{self, Duration}};

use chrono::DateTime;
use fltk::{app, prelude::*, window::Window};
use form::FormState;
use scale::{ScaleStatus, SerialCommand};
use spreadsheet::SpreadsheetWriter;

fn last_line(s: &str) -> Option<String> {
    let (mut line, _) = s.rsplit_once('\n')?;
    line = line.trim();
    let (_, line) = line.rsplit_once('\n')?;

    Some(line.to_string())
}

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Oxy);

    let (form_sender, form_reciever) = app::channel::<form::Message>();

    let mut window = Window::default()
            .with_size(800, 600)
            .with_label("Scale Log")
            .center_screen();

    let mut sl_form = form::ScaleLogForm::new(form_sender.clone()).unwrap();

    window.make_resizable(true);
    window.end();
    window.show();
    
    let (status_sender, status_reciever) = mpsc::channel::<scale::ScaleStatus>();
    let (serial_command_sender, serial_command_reciever) = mpsc::channel::<scale::SerialCommand>();

    thread::spawn(move || {
        // let port: Rc<RefCell<Option<Box<dyn serialport::SerialPort>>>> = Rc::new(RefCell::new(None));
        let mut port: Option<Box<dyn serialport::SerialPort>> = None;
        let mut started = false;
        let mut current_time = time::Instant::now();

        loop {
            // Process commands
            if let Ok(command) = serial_command_reciever.try_recv() {
                match command {
                    SerialCommand::OpenPort(name) => {
                        match serialport::new(name.as_str(), 9600).timeout(Duration::from_millis(50)).open() {
                            Ok(p) => {
                                port = Some(p);
                                status_sender.send(ScaleStatus::OpenSucceeded(name)).unwrap();
                            },
                            Err(_) => {
                                status_sender.send(ScaleStatus::OpenFailed(name)).unwrap();
                            }
                        }
                    },
                    SerialCommand::Start => {
                        if let Some(port) = &mut port {
                            port.clear(serialport::ClearBuffer::All).unwrap();
                        }
                        started = true;
                    },
                    SerialCommand::Stop => {
                        started = false;
                    }
                }
            }
            // Send weight
            if started && current_time.elapsed().as_millis() > 50 {
                if let Some(port) = &mut port {
                    // let mut buf = "".to_string();
                    let mut buf: Vec<u8> = vec![0; 64];
                    port.flush().unwrap();
                    match port.read_exact(buf.as_mut_slice()) {
                        Ok(_) => {
                            let buf = String::from_utf8_lossy(&buf);
                            let line = last_line(&buf);
                            if let Some(line) = line {
                                let weight = scale::ScaleWeight::from_str(&line);
                                if let Ok(weight) = weight {
                                    status_sender.send(ScaleStatus::Weight(weight)).unwrap();
                                }
                            }
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(_) => {
                            status_sender.send(ScaleStatus::Disconnected).unwrap();
                        }
                    }
                }
                current_time = time::Instant::now();
            }
        }
    });

    let mut sheet_writer = SpreadsheetWriter::new();

    while app::check() {
        if let Some(msg) = form_reciever.recv() {
            match msg {
                form::Message::OpenSerial => {
                    serial_command_sender.send(SerialCommand::OpenPort(sl_form.port_name())).unwrap();
                },
                form::Message::StartLog => {
                    sl_form.set_state(FormState::Started);
                    serial_command_sender.send(SerialCommand::Start).unwrap();
                    sheet_writer = SpreadsheetWriter::new();
                },
                form::Message::StopLog => {
                    sl_form.set_state(FormState::Stopped);
                    serial_command_sender.send(SerialCommand::Stop).unwrap();
                    sheet_writer.save();
                }
            }
        }

        if let Ok(status) = status_reciever.try_recv() {
            match status {
                ScaleStatus::OpenSucceeded(name) => {
                    sl_form.append_terminal(format!("Opened port '{}'\n", name).as_str());
                    sl_form.set_state(FormState::Stopped);
                },
                ScaleStatus::OpenFailed(name) => {
                    sl_form.append_terminal(format!("Failed to open port '{}'\n", name).as_str());
                },
                ScaleStatus::Weight(weight) => {
                    let datetime: DateTime<chrono::offset::Local> = weight.time.into();
                    let stable = if weight.stable {
                        "Stable"
                    }
                    else {
                        "Unstable"
                    };
                    sl_form.append_terminal(&format!("[{}] {} {} {}\n", datetime.format("%m-%d-%Y %H:%M:%S"), stable, weight.value, weight.unit));
                    sheet_writer.append(datetime, weight.value, weight.unit);
                },
                ScaleStatus::Disconnected => {
                    sl_form.append_terminal("Port disconnected\n");
                    sl_form.set_state(FormState::NoPortOpen);
                },
            }
        }
    }
}
