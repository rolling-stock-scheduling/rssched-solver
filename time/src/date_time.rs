use std::fmt;
use std::ops::Add;
use std::ops::Sub;

use super::{duration::DurationLength, Duration};

// Important: Leap year are integrated. But no daylight-saving.

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)] // care the ordering of the variants is important
pub enum DateTime {
    Earliest, // always earlier than all TimePoints
    Point(TimePoint),
    Latest, // always later than all TimePoints
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)] // care the ordering of attributes is important
pub struct TimePoint {
    year: u32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl DateTime {
    pub fn new(string: &str) -> DateTime {
        //"2009-06-15T13:45:13" or "2009-4-15T12:10"
        let shortened = string.replace('Z', "");
        let splitted: Vec<&str> = shortened.split(&['T', '-', ' ', ':'][..]).collect();
        let len = splitted.len();
        assert!(len <= 6 && len >= 5, "Wrong time format.");

        let year: u32 = splitted[0].parse().expect("Error at year.");
        let month: u8 = splitted[1].parse().expect("Error at month.");
        assert!(month <= 12 && month >= 1, "Wrong month format.");
        let day: u8 = splitted[2].parse().expect("Error at day.");
        assert!(
            day <= TimePoint::get_days_of_month(year, month) && day >= 1,
            "Wrong day format."
        );
        let hour: u8 = splitted[3].parse().expect("Error at hour.");
        assert!(hour <= 24, "Wrong hour format.");
        let minute: u8 = splitted[4].parse().expect("Error at minute.");
        assert!(minute < 60, "Wrong minute format.");
        let second: u8 = if len == 6 {
            splitted[5].parse().expect("Error at second.")
        } else {
            0
        };

        DateTime::Point(TimePoint {
            year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }
}

impl DateTime {
    pub fn as_iso(&self) -> String {
        match self {
            DateTime::Earliest => String::from("EARLIEST"),
            DateTime::Point(t) => format!(
                "{:#04}-{:#02}-{:#02}T{:#02}:{:#02}:{:#02}",
                t.year, t.month, t.day, t.hour, t.minute, t.second
            ),
            DateTime::Latest => String::from("LATEST"),
        }
    }
}

impl Add<Duration> for DateTime {
    type Output = Self;

    fn add(self, other: Duration) -> Self {
        match other {
            Duration::Infinity => DateTime::Latest, //note that Earliest + Infinity = Latest
            Duration::Length(l) => match self {
                DateTime::Earliest => DateTime::Earliest,
                DateTime::Point(t) => DateTime::Point(t + l),
                DateTime::Latest => DateTime::Latest,
            },
        }
    }
}

impl Sub for DateTime {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        assert!(other <= self, "Cannot subtract {} from {}, as it is a later point in time (no negative durations allowed)", other, self);
        match self {
            DateTime::Earliest => {
                Duration::Length(DurationLength {
                    hours: 0,
                    minutes: 0,
                    seconds: 0,
                }) // Earliest - Earliest
            }
            DateTime::Latest => {
                if other == DateTime::Latest {
                    Duration::Length(DurationLength {
                        hours: 0,
                        minutes: 0,
                        seconds: 0,
                    }) // Latest - Latest
                } else {
                    Duration::Infinity // Latest - (something not Latest)
                }
            }
            DateTime::Point(l1) => {
                match other {
                    DateTime::Earliest => Duration::Infinity, // Length - Earliest
                    DateTime::Point(l2) => l1 - l2,
                    _ => panic!("This can never be reached"),
                }
            }
        }
    }
}

impl Sub<Duration> for DateTime {
    type Output = DateTime;

    fn sub(self, other: Duration) -> DateTime {
        match self {
            DateTime::Earliest => DateTime::Earliest,
            DateTime::Latest => {
                if other == Duration::Infinity {
                    panic!("Cannot subtract Infinity from Latest");
                } else {
                    DateTime::Latest
                }
            }
            DateTime::Point(t) => match other {
                Duration::Infinity => DateTime::Earliest,
                Duration::Length(d) => DateTime::Point(t - d),
            },
        }
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DateTime::Earliest => write!(f, "Earliest"),
            DateTime::Point(t) => write!(f, "{}", t),
            DateTime::Latest => write!(f, "Latest"),
        }
    }
}

////////////////////////////////////////////////////////////////////
////////////////////////// TimePoint ///////////////////////////////
////////////////////////////////////////////////////////////////////

// useful static functions:
impl TimePoint {
    // given a date, that is invalid (too many days for the month) compute the correct date
    fn correct_date(year: u32, month: u8, day: u32) -> (u32, u8, u8) {
        let days_of_month = TimePoint::get_days_of_month(year, month) as u32;

        if day <= days_of_month {
            return (year, month, day as u8);
        }
        let new_day = day - days_of_month;
        let mut new_month = month + 1;
        let mut new_year = year;
        if new_month > 12 {
            new_year += 1;
            new_month -= 12;
        }

        TimePoint::correct_date(new_year, new_month, new_day)
    }

    fn get_days_of_month(year: u32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }
}

impl Sub for TimePoint {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        assert!(other <= self, "Cannot subtract {} from {}, as it is a later point in time (no negative durations allowed)", other, self);

        let mut hours = (if self.hour >= other.hour {
            self.hour - other.hour
        } else {
            self.hour + 24 - other.hour
        }) as u32;
        let mut minutes = if self.minute >= other.minute {
            self.minute - other.minute
        } else {
            hours = if hours > 0 { hours - 1 } else { 23 }; // subtract one of the hours
            self.minute + 60 - other.minute
        };
        let seconds = if self.second >= other.second {
            self.second - other.second
        } else {
            // subtract one of the minutes
            minutes = if minutes > 0 {
                minutes - 1
            } else {
                // subtract one of the hours
                hours = if hours > 0 { hours - 1 } else { 23 };
                59
            };
            self.second + 60 - other.second
        };

        let mut temp_date = other
            + DurationLength {
                hours,
                minutes,
                seconds,
            };
        while self != temp_date {
            let days_diff = if self.day > temp_date.day {
                self.day - temp_date.day
            } else {
                self.day + TimePoint::get_days_of_month(temp_date.year, temp_date.month)
                    - temp_date.day
            };
            let hours_diff = 24 * days_diff as u32;
            temp_date = temp_date
                + DurationLength {
                    hours: hours_diff,
                    minutes: 0,
                    seconds: 0,
                };
            hours += hours_diff;
        }

        Duration::Length(DurationLength {
            hours,
            minutes,
            seconds,
        })
    }
}

impl Add<DurationLength> for TimePoint {
    type Output = Self;

    fn add(self, other: DurationLength) -> Self {
        let sum_of_seconds = self.second + other.seconds;
        let second = sum_of_seconds % 60;
        let sum_of_minutes = self.minute + other.minutes + (sum_of_seconds / 60) as u8;
        let minute = sum_of_minutes % 60;
        let sum_of_hours: u32 = self.hour as u32 + other.hours + (sum_of_minutes / 60) as u32;
        let hour = (sum_of_hours % 24) as u8;
        let sum_of_days: u32 = self.day as u32 + sum_of_hours / 24;

        let (year, month, day) = TimePoint::correct_date(self.year, self.month, sum_of_days);

        TimePoint {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }
}

impl Sub<DurationLength> for TimePoint {
    type Output = TimePoint;

    fn sub(self, other: DurationLength) -> Self {
        let mut year = self.year;
        let mut month = self.month;
        let mut day = self.day as u32;
        let mut hour = self.hour as u32;
        let mut minute = self.minute;
        let mut second = self.second;

        let mut other_days = 0;
        let mut other_hours = other.hours;
        let mut other_minutes = other.minutes;

        if other.seconds > second {
            second += 60;
            other_minutes += 1;
        }
        second -= other.seconds;

        if other_minutes > minute {
            minute += 60;
            other_hours += 1;
        }
        minute -= other_minutes;

        if other_hours > hour {
            let day_diff: u32 = (other_hours - hour) / 24 + 1;
            hour += day_diff * 24;
            other_days += day_diff;
        }
        hour -= other_hours;

        while other_days > day {
            if month == 1 {
                year -= 1;
                month = 12;
            } else {
                month -= 1;
            }
            day += TimePoint::get_days_of_month(year, month) as u32;
        }
        day -= other_days;

        TimePoint {
            year,
            month,
            day: day as u8,
            hour: hour as u8,
            minute,
            second,
        }
    }
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.second > 0 {
            write!(
                f,
                "{:02}.{:02}.{}_{:02}:{:02}:{:02}",
                self.day, self.month, self.year, self.hour, self.minute, self.second
            )
        } else {
            write!(
                f,
                "{:02}.{:02}.{}_{:02}:{:02}",
                self.day, self.month, self.year, self.hour, self.minute
            )
        }
    }
}
