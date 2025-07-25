use std::{
    fs::Metadata,
    io, mem,
    time::{Duration, SystemTime},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Time {
    duration_since_epoch: Duration,
    offset: i64,
}

impl From<SystemTime> for Time {
    fn from(value: SystemTime) -> Self {
        let duration_since_epoch = value
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(1));
        Self {
            duration_since_epoch,
            offset: Self::get_local_timezone_offset(duration_since_epoch.as_secs() as i64),
        }
    }
}

impl Time {
    pub fn from_created(metadata: &Metadata) -> io::Result<Self> {
        let created = metadata.created()?;
        Ok(Self::from(created))
    }
    pub fn from_modified(metadata: &Metadata) -> io::Result<Self> {
        let modified = metadata.modified()?;
        Ok(Self::from(modified))
    }
    #[cfg(unix)]
    fn get_unix_timezone_offset(duration_since_epoch: i64) -> i64 {
        use libc::{localtime_r, time_t, tm};

        unsafe {
            let timestamp = duration_since_epoch as time_t;
            let mut tm_result: tm = mem::zeroed();

            if !localtime_r(&timestamp, &mut tm_result).is_null() {
                tm_result.tm_gmtoff
            } else {
                0
            }
        }
    }
    #[cfg(windows)]
    fn get_windows_timezone_offset() -> i64 {
        use winapi::um::timezoneapi::{GetTimeZoneInformation, TIME_ZONE_ID_INVALID};

        unsafe {
            let mut tz_info = mem::zeroed();
            if GetTimeZoneInformation(&mut tz_info) != TIME_ZONE_ID_INVALID {
                -(tz_info.Bias as i64) * 60
            } else {
                0
            }
        }
    }
    fn get_local_timezone_offset(duration_since_epoch: i64) -> i64 {
        #[cfg(unix)]
        return Self::get_unix_timezone_offset(duration_since_epoch);
        #[cfg(windows)]
        return Self::get_windows_timezone_offset();
        #[cfg(not(any(unix, windows)))]
        0
    }
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
    fn get_days_in_year(year: i32) -> i32 {
        if Self::is_leap_year(year) { 366 } else { 365 }
    }
    fn get_days_in_month(month: u32, year: i32) -> i32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }
    fn _get_day_of_week(&self) -> u32 {
        let (year, month, day) = self.to_calendar_date();
        let (m, y) = if month < 3 {
            (month + 12, year - 1)
        } else {
            (month, year)
        };

        let k = y % 100;
        let j = y / 100;
        let h = (day as i32 + (13 * (m as i32 + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
        ((h + 5) % 7) as u32
    }
    fn secs(&self) -> u64 {
        self.duration_since_epoch.as_secs()
    }
    fn to_calendar_date(&self) -> (i32, u32, u32) {
        let secs = self.secs() + self.offset as u64;
        let mut days = secs as i32 / 86400;
        // let rem_secs = secs % 86400;

        let mut year = 1970;
        let mut days_in_year = Self::get_days_in_year(year);

        while days >= days_in_year {
            days -= days_in_year;
            year += 1;
            days_in_year = Self::get_days_in_year(year);
        }

        while days < 0 {
            year -= 1;
            days_in_year = Self::get_days_in_year(year);
            days += days_in_year;
        }

        let mut month = 1;
        let mut days_in_month = Self::get_days_in_month(month, year);

        while days >= days_in_month {
            days -= days_in_month;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            days_in_month = Self::get_days_in_month(month, year);
        }

        let day = (days + 1) as u32;

        (year, month, day)
    }
    fn to_time_parts(&self) -> (u32, u32, u32) {
        let secs = (self.secs() as u32 + self.offset as u32) % 86400;

        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        (hours, minutes, seconds)
    }
    pub fn format(&self) -> String {
        let (_year, month, day) = self.to_calendar_date();
        let (hours, minutes, _seconds) = self.to_time_parts();
        // let day_of_week = self.get_day_of_week();

        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let month_str = months.get((month - 1) as usize).unwrap_or(&"???");

        format!("{month_str} {day:>2} {hours:02}:{minutes:02}")
    }
}
