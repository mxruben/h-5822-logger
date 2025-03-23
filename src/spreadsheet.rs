use std::time;

use chrono::{DateTime, Local};
use rust_xlsxwriter::workbook::Workbook;

use crate::scale::ScaleUnit;

pub struct SpreadsheetWriter {
    workbook: Workbook,
    row: u32
}

impl SpreadsheetWriter {
    pub fn new() -> Self {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        worksheet.write(0, 0, "Time").unwrap();
        worksheet.write(0, 1, "Weight").unwrap();
        worksheet.write(0, 2, "Unit").unwrap();

        Self {
            workbook,
            row: 1,
        }
    }

    pub fn append(&mut self, time: DateTime<Local>, weight: f32, unit: ScaleUnit) {
        let worksheet = self.workbook.worksheet_from_index(0).unwrap();

        worksheet.write(self.row, 0, time.format("%m-%d-%Y %H:%M:%S").to_string()).unwrap();
        worksheet.write(self.row, 1, weight).unwrap();
        worksheet.write(self.row, 2, unit.to_string()).unwrap();

        self.row += 1;
    }

    pub fn save(&mut self) {
        let time = time::SystemTime::now();
        let datetime: DateTime<Local> = time.into();
        self.workbook.save(format!("log {}.xlsx", datetime.format("%m-%d-%Y %H.%M.%S"))).unwrap();
    }
}
