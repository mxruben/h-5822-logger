mod form;
mod scale;
mod spreadsheet;

use chrono::DateTime;
use fltk_theme::{ThemeType, WidgetTheme};
use fltk::app;
use form::FormState;
use scale::{ScaleLogger, ScaleStatus};
use spreadsheet::SpreadsheetWriter;

fn main() {
    let _app = app::App::default();
    let widget_theme = WidgetTheme::new(ThemeType::Metro);
    widget_theme.apply();

    let (form_sender, form_reciever) = app::channel::<form::FormMessage>();

    let mut sl_form = form::ScaleLogForm::new(form_sender.clone()).unwrap();

    let scale_logger = ScaleLogger::new();
    let mut sheet_writer = SpreadsheetWriter::new();

    while app::check() {
        if let Some(msg) = form_reciever.recv() {
            match msg {
                form::FormMessage::OpenSerial => {
                    scale_logger.open(sl_form.port_name()).unwrap()
                },
                form::FormMessage::StartLog => {
                    sl_form.set_state(FormState::Started);
                    scale_logger.start_log(sl_form.log_frequency()).unwrap();
                    sheet_writer = SpreadsheetWriter::new();
                },
                form::FormMessage::StopLog => {
                    sl_form.set_state(FormState::Stopped);
                    scale_logger.stop_log().unwrap();
                    sheet_writer.save();
                }
            }
        }

        if let Ok(status) = scale_logger.try_status() {
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
