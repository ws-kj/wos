use lazy_static::lazy_static;
use spin::Mutex;
use crate::io;
use alloc::string::{ToString, String};
pub const CURRENT_YEAR: usize = 2020;
pub const CMOS_ADDR: u16 = 0x70;
pub const CMOS_DATA: u16 = 0x71;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct RtcHandler {
    century_reg: u8,
    century: u8,
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: usize,
}

lazy_static! {
    pub static ref RTC: Mutex<RtcHandler> = Mutex::new(Default::default());
}

impl RtcHandler {
    fn get_update_in_progress_flag(&mut self) -> u8 {
        unsafe {
            io::outb(CMOS_ADDR, 0x0A);
            //println!("{:?}", io::inb(CMOS_DATA) & 0x80);
            io::inb(CMOS_DATA) & 0x80
        }
    }

    fn get_rtc_reg(&mut self, reg: u8) -> u8 {
        unsafe {
            io::outb(CMOS_ADDR, reg);
            io::inb(CMOS_DATA)
        }
    }

    pub fn read_rtc(&mut self) {
        let mut last_rtc: RtcHandler = Default::default();
        let mut reg_b: u8 = 0;

        while self.get_update_in_progress_flag() == 1 {}
        self.second = self.get_rtc_reg(0x00);
        self.minute = self.get_rtc_reg(0x02);
        self.hour   = self.get_rtc_reg(0x04);
        self.day    = self.get_rtc_reg(0x07);
        self.month  = self.get_rtc_reg(0x08);
        self.year   = self.get_rtc_reg(0x09) as usize;
        if self.century_reg != 0 {
            self.century = self.get_rtc_reg(self.century_reg);
        }
        //evil do-while trick
        while {
            last_rtc.second = self.second;
            last_rtc.minute = self.minute;
            last_rtc.hour   = self.hour;
            last_rtc.day    = self.day;
            last_rtc.month  = self.month;
            last_rtc.year   = self.year;
            last_rtc.century = self.century;

            while self.get_update_in_progress_flag() == 1 {}
            self.second = self.get_rtc_reg(0x00);
            self.minute = self.get_rtc_reg(0x02);
            self.hour   = self.get_rtc_reg(0x04);
            self.day    = self.get_rtc_reg(0x07);
            self.month  = self.get_rtc_reg(0x08);
            self.year   = self.get_rtc_reg(0x09) as usize;
            if self.century_reg != 0 {
                self.century = self.get_rtc_reg(self.century_reg);
            }

            (last_rtc.second != self.second) || 
                (last_rtc.minute != self.minute) ||
                (last_rtc.hour != self.hour) ||
                (last_rtc.day != self.day) ||
                (last_rtc.month != self.month) || 
                (last_rtc.year != self.year) ||
                (last_rtc.century != self.century)
        } {} //evil bad no-good trick to make a do while loop

        reg_b = self.get_rtc_reg(0x0B);
        //convert BCD to binary
        if (reg_b & 0x04) == 0 {
            self.second = (self.second & 0x0F) + ((self.second / 16) * 10);
            self.minute = (self.minute & 0x0F) + ((self.minute / 16) * 10);
            self.hour   = ((self.hour & 0x0F) + (((self.hour & 0x70) / 16) * 10)) | (self.hour & 0x80);
            self.day    = (self.day & 0x0F) + ((self.day / 16) * 10);
            self.month  = (self.month & 0x0F) + ((self.month / 16) * 10);
            self.year   = (self.year & 0xF) + ((self.year / 16) * 10);
            if self.century_reg != 0 {
                self.century = (self.century & 0x0F) + ((self.century / 16) * 10);
            }
        }

        //12hr clock -> 24hr clock
        if (reg_b & 0x02) == 0 && (self.hour & 0x80) == 1 {
            self.hour = ((self.hour & 0x7F) + 12) % 24;
        }

        if self.century_reg != 0 {
            self.year += (self.century * 100) as usize;
        } else {
            self.year += (CURRENT_YEAR / 100) * 100;
            if self.year < CURRENT_YEAR {
                self.year += 100;
            }
        }
    }

    pub fn get_time(&mut self) -> String {
        self.read_rtc();
        let mut r = self.hour.to_string();
        r.push_str(&String::from(":"));
        r.push_str(&self.minute.to_string());
        r.push_str(&String::from(":"));
        r.push_str(&self.second.to_string());
        r
    }

    pub fn get_date(&mut self) -> String {
        self.read_rtc();
        let mut r = self.year.to_string();
        r.push_str(&String::from("-"));
        r.push_str(&self.month.to_string());
        r.push_str(&String::from("-"));
        r.push_str(&self.day.to_string());
        r
    }

    pub fn get_datetime(&mut self) -> String {
        let mut r = self.get_date();
        r.push_str(&String::from(" "));
        r.push_str(&self.get_time());
        r.push_str(&String::from(" UTC"));
        r
    }
}
