use std::{thread, time, fmt, io};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleUnit {
    Pounds,
    Ounces,
    Grams,
    Kilograms
}

impl fmt::Display for ScaleUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaleUnit::Pounds => write!(f, "lb"),
            ScaleUnit::Ounces => write!(f, "oz"),
            ScaleUnit::Grams => write!(f, "g"),
            ScaleUnit::Kilograms => write!(f, "kg"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleWeight {
    pub unit: ScaleUnit,
    pub value: f32,
    pub stable: bool,
    pub time: time::SystemTime
}

#[derive(Debug)]
pub enum WeightConversionError {
    InvalidString,
    InvalidStable,
    InvalidWeight,
    InvalidUnit
}

impl ScaleWeight {
    pub fn from_str(s: &str) -> Result<Self, WeightConversionError> {
        if s.len() < 16 {
            return Err(WeightConversionError::InvalidString)
        }

        let stable = match &s[..2] {
            "ST" => {
                true
            },
            "US" => {
                false
            },
            _ => {
                return Err(WeightConversionError::InvalidStable)
            }
        };

        let value_str = s[6..14].trim();
        let value: f32 = match value_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(WeightConversionError::InvalidWeight)
            }
        };

        let unit = match s[15..].trim() {
            "oz" => {
                ScaleUnit::Ounces
            },
            "lb" => {
                ScaleUnit::Pounds
            },
            "g" => {
                ScaleUnit::Grams
            },
            "kg" => {
                ScaleUnit::Kilograms
            }
            _ => {
                return Err(WeightConversionError::InvalidUnit)
            }
        };

        Ok(Self {
            unit,
            value,
            stable,
            time: time::SystemTime::now()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScaleStatus {
    OpenSucceeded(String),
    OpenFailed(String),
    Weight(ScaleWeight),
    Disconnected
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScaleLoggerCommand {
    OpenPort(String),
    StartLog(u128),
    StopLog,
}

pub struct ScaleLogger {
    status_reciever: crossbeam_channel::Receiver<ScaleStatus>,
    command_sender: crossbeam_channel::Sender<ScaleLoggerCommand>
}

fn last_line(s: &str) -> Option<String> {
    let (mut line, _) = s.rsplit_once('\n')?;
    line = line.trim();
    let (_, line) = line.rsplit_once('\n')?;

    Some(line.to_string())
}

impl ScaleLogger {
    pub fn new() -> Self {
        let (status_sender, status_reciever) = crossbeam_channel::unbounded::<ScaleStatus>();
        let (command_sender, command_reciever) = crossbeam_channel::unbounded::<ScaleLoggerCommand>();
        let scale_logger = Self {
            status_reciever,
            command_sender
        };
        scale_logger.spawn(status_sender, command_reciever);
        scale_logger
    }

    pub fn open(&self, name: String) -> Result<(), crossbeam_channel::SendError<ScaleLoggerCommand>> {
        self.command_sender.send(ScaleLoggerCommand::OpenPort(name))?;
        Ok(())
    }

    pub fn start_log(&self, frequency: u128) -> Result<(), crossbeam_channel::SendError<ScaleLoggerCommand>> {
        self.command_sender.send(ScaleLoggerCommand::StartLog(frequency))?;
        Ok(())
    }

    pub fn stop_log(&self) -> Result<(), crossbeam_channel::SendError<ScaleLoggerCommand>> {
        self.command_sender.send(ScaleLoggerCommand::StopLog)?;
        Ok(())
    }

    pub fn try_status(&self) -> Result<ScaleStatus, crossbeam_channel::TryRecvError> {
        self.status_reciever.try_recv()
    }

    fn spawn(&self, status_sender: crossbeam_channel::Sender<ScaleStatus>, command_reciever: crossbeam_channel::Receiver<ScaleLoggerCommand>) {
        thread::spawn(move || {
            let mut port: Option<Box<dyn serialport::SerialPort>> = None;
            let mut started = false;
            let mut current_time = time::Instant::now();
            let mut log_frequency: u128 = 500;
    
            loop {
                // Process commands
                if let Ok(command) = command_reciever.try_recv() {
                    match command {
                        ScaleLoggerCommand::OpenPort(name) => {
                            match serialport::new(name.as_str(), 9600).timeout(time::Duration::from_millis(50)).open() {
                                Ok(p) => {
                                    port = Some(p);
                                    status_sender.send(ScaleStatus::OpenSucceeded(name)).unwrap();
                                },
                                Err(_) => {
                                    status_sender.send(ScaleStatus::OpenFailed(name)).unwrap();
                                }
                            }
                        },
                        ScaleLoggerCommand::StartLog(frequency) => {
                            // if let Some(port) = &mut port {
                            //     port.clear(serialport::ClearBuffer::All).unwrap();
                            // }
                            log_frequency = frequency;
                            started = true;
                        },
                        ScaleLoggerCommand::StopLog => {
                            started = false;
                        }
                    }
                }
                // Send weight
                if started && current_time.elapsed().as_millis() > log_frequency {
                    if let Some(port) = &mut port {
                        port.clear(serialport::ClearBuffer::All).unwrap();
                        let mut buf: Vec<u8> = vec![0; 64];
                        port.flush().unwrap();
                        match port.read_exact(buf.as_mut_slice()) {
                            Ok(_) => {
                                let buf = String::from_utf8_lossy(&buf);
                                let line = last_line(&buf);
                                if let Some(line) = line {
                                    let weight = ScaleWeight::from_str(&line);
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
    }
}
