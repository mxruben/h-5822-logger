use fltk::group;
#[allow(deprecated)]
use fltk::{app, button::Button, frame::Frame, group::Grid, input::Input, prelude::*, text::SimpleTerminal};

#[derive(Debug, Clone)]
pub enum Message {
    OpenSerial,
    StartLog,
    StopLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormState {
    NoPortOpen,
    Stopped,
    Started
}

#[allow(deprecated)]
pub struct ScaleLogForm {
    terminal: SimpleTerminal,

    entry_port: Input,
    button_open: Button,
    button_start: Button,
    button_stop: Button,

    state: FormState,
}

impl ScaleLogForm {
    pub fn new(sender: app::Sender<Message>) -> Result<Self, FltkError> {  
        let mut grid = Grid::default_fill();
        
        let mut entry_port = Input::default();

        let mut button_open = Button::default()
            .with_label("Open");
        button_open.emit(sender.clone(), Message::OpenSerial);

        let mut button_start = Button::default()
            .with_label("Start");
        button_start.emit(sender.clone(), Message::StartLog);
        button_start.deactivate();

        let mut button_stop = Button::default()
            .with_label("Stop");
        button_stop.emit(sender.clone(), Message::StopLog);
        button_stop.deactivate();

        #[allow(deprecated)]
        let mut terminal = SimpleTerminal::default();
        terminal.set_stay_at_bottom(true);
        terminal.append("No serial port open\n");

        let mut entry_port_label = Frame::default()
            .with_label("Port: ");

        grid.show_grid(false);
        grid.set_layout(20, 9);
        grid.set_gap(3, 3);
        grid.set_margins(3, 3, 3, 3);

        grid.set_widget(&mut entry_port_label, 0, 0)?;
        grid.set_widget(&mut entry_port, 0, 1..3)?;
        grid.set_widget(&mut button_open, 0, 3)?;
        grid.set_widget(&mut button_start, 0, 4)?;
        grid.set_widget(&mut button_stop, 0, 5)?;
        grid.set_widget(&mut terminal, 1..20, 0..9)?;

        grid.end();

        let mut scale_log_form = Self {
            terminal,
            entry_port,
            button_open,
            button_start,
            button_stop,

            state: FormState::NoPortOpen
        };

        scale_log_form.set_state(FormState::NoPortOpen);

        Ok(scale_log_form)
    }

    pub fn append_terminal(&mut self, s: &str) {
        self.terminal.append(s);
        self.terminal.redraw();
    }

    pub fn port_name(&self) -> String {
        self.entry_port.value()
    }

    pub fn set_state(&mut self, state: FormState) {
        match state {
            FormState::NoPortOpen => {
                self.button_start.deactivate();
                self.button_stop.deactivate();
            },
            FormState::Started => {
                self.button_start.deactivate();
                self.button_stop.activate();
                self.button_open.deactivate();
            },
            FormState::Stopped => {
                self.button_open.activate();
                self.button_start.activate();
                self.button_stop.deactivate();
            }
        };
        self.state = state;
    }
}
