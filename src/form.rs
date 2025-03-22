use fltk::{enums::Color, input::IntInput, misc::InputChoice, window};
#[allow(deprecated)]
use fltk::{app, button::Button, frame::Frame, group::{self, Flex}, input::Input, prelude::*, text::SimpleTerminal, enums};

#[derive(Debug, Clone)]
pub enum FormMessage {
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

    input_log_frequency: IntInput,
    choice_log_frequency_unit: InputChoice,

    state: FormState,
}

impl ScaleLogForm {
    pub fn new(sender: app::Sender<FormMessage>) -> Result<Self, FltkError> {
        let mut window = window::Window::default()
            .with_size(800, 600)
            .with_label("Scale Log")
            .center_screen();

        let mut flex_form = Flex::default_fill()
            .with_type(group::FlexType::Column);
        flex_form.set_pad(0);

        let mut flex_button = Flex::default()
            .with_type(group::FlexType::Row);
        flex_button.set_spacing(3);
        flex_button.set_margin(3);
        flex_form.fixed(&flex_button, 40);
        
        let label_entry_port = Frame::default()
            .with_label("Port: ");
        flex_button.fixed(&label_entry_port, 40);

        let entry_port = Input::default();
        flex_button.fixed(&entry_port, 100);

        let mut button_open = Button::default()
            .with_label("Open");
        button_open.emit(sender.clone(), FormMessage::OpenSerial);
        flex_button.fixed(&button_open, 50);

        let mut button_start = Button::default()
            .with_label("Start");
        button_start.emit(sender.clone(), FormMessage::StartLog);
        button_start.deactivate();
        flex_button.fixed(&button_start, 50);

        let mut button_stop = Button::default()
            .with_label("Stop");
        button_stop.emit(sender.clone(), FormMessage::StopLog);
        button_stop.deactivate();
        flex_button.fixed(&button_stop, 50);

        let label_log_frequency = Frame::default()
            .with_label("Log frequency: ");
        flex_button.fixed(&label_log_frequency, label_log_frequency.measure_label().0);

        let mut input_log_frequency = IntInput::default();
        input_log_frequency.set_value("500");
        input_log_frequency.set_maximum_size(9);
        flex_button.fixed(&input_log_frequency, 40);

        let mut choice_log_frequency_unit = InputChoice::default();
        choice_log_frequency_unit.set_down_frame(enums::FrameType::ShadowFrame);
        choice_log_frequency_unit.input().set_readonly(true);

        choice_log_frequency_unit.add("ms");
        choice_log_frequency_unit.add("s");
        choice_log_frequency_unit.add("min");
        choice_log_frequency_unit.set_value("ms");

        flex_button.fixed(&choice_log_frequency_unit, 60);

        flex_button.end();

        #[allow(deprecated)]
        let mut terminal = SimpleTerminal::default_fill();
        terminal.set_stay_at_bottom(true);
        terminal.append("No serial port open\n");

        flex_form.end();

        window.make_resizable(true);
        window.end();
        window.show();

        let mut scale_log_form = Self {
            terminal,
            entry_port,
            button_open,
            button_start,
            button_stop,

            input_log_frequency,
            choice_log_frequency_unit,

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

    /// Returns frequency in milliseconds.
    pub fn log_frequency(&self) -> u128 {
        let frequency: u128 = self.input_log_frequency.value().parse().unwrap();
        let multiplier = if let Some(unit) = self.choice_log_frequency_unit.value() {
            match unit.as_str() {
                "ms" => 1,
                "s" => 1_000,
                "min" => 60_000,
                _ => 1
            }
        }
        else {
            1
        };
        
        frequency * multiplier
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
