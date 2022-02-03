use std::fmt;
use std::ops::Add;


#[derive(Copy, Clone, PartialEq, PartialOrd)] // care the ordering of the variants is important
pub(crate) enum Time {
    Earliest, // always earlier than all DateTimes
    Time(DateTime),
    Latest, // always later than all DateTimes
}

impl Time {
    pub fn new(string: &str) -> Time { //"2009-06-15T13:45" or "06-15 13:45" (fills year 0)
        let splitted: Vec<&str> = string.split(&['T','-',' ',':'][..]).collect();
        let len = splitted.len();
        assert!(len <= 5 && len >= 5, "Wrong time format.");

        let year: u32 = splitted[0].parse().expect("Error at year.");
        let month: u8 = splitted[1].parse().expect("Error at month.");
        assert!(month <= 12 && month >= 1, "Wrong month format.");
        let day: u8 = splitted[2].parse().expect("Error at day.");
        assert!(day <= DateTime::get_days_of_month(year,month) && day >= 1, "Wrong day format.");
        let hour: u8 = splitted[3].parse().expect("Error at hour.");
        assert!(hour <= 24, "Wrong hour format.");
        let minute: u8 = splitted[4].parse().expect("Error at minute.");
        assert!(minute < 60, "Wrong minute format.");

        Time::Time(DateTime{
            year,
            month,
            day,
            hour,
            minute})
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Time::Earliest => write!(f, "Earliest"),
            Time::Time(t) => write!(f, "{:02}.{:02}.{}_{:02}:{:02}", t.day, t.month, t.year, t.hour, t.minute),
            Time::Latest => write!(f, "Latest")
        }
    }
}


#[derive(Copy, Clone, PartialEq, PartialOrd)] // care the ordering of attributes is important
pub(crate) struct DateTime {
    year: u32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8
}

impl DateTime {
    // given a date, that is invalid (too many days for the month) compute the correct date
    fn correct_date(year: u32, month: u8, day: u32) -> (u32, u8, u8) {
        let days_of_month = DateTime::get_days_of_month(year, month) as u32;

        if day <= days_of_month {
            return (year, month, day as u8);
        }
        let new_day = day - days_of_month;
        let mut new_month = month + 1;
        let mut new_year = year;
        if new_month > 12 {
            new_year += 1;
            new_month = new_month - 12;
        }

        DateTime::correct_date(new_year, new_month, new_day)

    }

    fn get_days_of_month(year: u32, month: u8) -> u8 {
         match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {29} else {28},
            _ => 0
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) struct Duration {
    hours: u32,
    minutes: u8
}

impl Duration {
    pub fn new(string: &str) -> Duration { //"13:45"
        let splitted: Vec<&str> = string.split(&[':'][..]).collect();
        assert!(splitted.len() == 2, "Wrong time format.");

        let hours: u32 = splitted[0].parse().expect("Error at hour.");
        let minutes: u8 = splitted[1].parse().expect("Error at minute.");
        assert!(minutes < 60, "Wrong minute format.");

        Duration{
            hours,
            minutes
        }
    }

}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hours, self.minutes)
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let sum_of_minutes = self.minutes + other.minutes;
        let minutes = sum_of_minutes % 60;
        let hours = self.hours + other.hours + (sum_of_minutes/60) as u32;
        Duration{
            hours,
            minutes
        }
    }
}

impl Add<Duration> for DateTime {
    type Output = Self;

    fn add(self, other: Duration) -> Self {
        let sum_of_minutes = self.minute + other.minutes;
        let minute = sum_of_minutes % 60;
        let sum_of_hours: u32 = self.hour as u32 + other.hours + (sum_of_minutes/60) as u32;
        let hour = (sum_of_hours % 24) as u8;
        let sum_of_days: u32 = self.day as u32 + sum_of_hours/24;

        let (year,month,day) = DateTime::correct_date(self.year, self.month, sum_of_days);


        DateTime{
            year,
            month,
            day,
            hour,
            minute
        }
    }
}

impl Add<Duration> for Time {
    type Output = Self;

    fn add(self, other: Duration) -> Self {
        match self {
            Time::Earliest => Time::Earliest,
            Time::Time(dt) => Time::Time(dt + other),
            Time::Latest => Time::Latest
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sum_up_duration() {
        let dur1 = Duration::new("5000:40");
        let dur2 = Duration::new("00:46");
        let sum = Duration::new("5001:26");
        assert!(dur1 + dur2 == sum, "Duration does not sum up correctly. dur1: {} + dur2: {} is {}; but should be {}", dur1, dur2, dur1 + dur2, sum);
    }

    #[test]
    fn add_duration_to_time_no_leap_year() {
        let time = Time::new("1999-2-28T23:40");
        let dur = Duration::new("48:46");
        let sum = Time::new("1999-3-3T00:26");
        assert!(time + dur == sum, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be {}", time, dur, time + dur, sum);
    }

    #[test]
    fn add_duration_to_time_leap_year() {
        let time = Time::new("2000-2-28T23:40");
        let dur = Duration::new("48:46");
        let sum = Time::new("2000-3-2T00:26");
        assert!(time + dur == sum, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be {}", time, dur, time + dur, sum);
    }

    #[test]
    fn add_long_duration_to_time() {
        let time = Time::new("1-01-01T00:00"); // jesus just got one year old ;)
        let dur = Duration::new("10000000:00");
        let sum = Time::new("1141-10-18T16:00");
        assert!(time + dur == sum, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be {}", time, dur, time + dur, sum);

    }
    #[test]
    fn add_duration_to_earliest_latest() {
        let earliest = Time::Earliest; // jesus just got one year old ;)
        let dur = Duration::new("50:00");
        assert!(earliest + dur == Time::Earliest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Earliest", earliest, dur, earliest + dur);

        let latest = Time::Latest; // jesus just got one year old ;)
        let dur = Duration::new("50:00");
        assert!(latest + dur == Time::Latest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Latest", latest, dur, latest + dur);

    }
}
