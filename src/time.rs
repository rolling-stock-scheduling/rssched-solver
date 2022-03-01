use std::fmt;
use std::ops::Add;
use std::ops::Sub;


// Important: Leap year are integrated. But no daylight-saving.

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)] // care the ordering of the variants is important
pub(crate) enum Time {
    Earliest, // always earlier than all TimePoints
    Point(TimePoint),
    Latest, // always later than all TimePoints
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)] // care the ordering of attributes is important
pub(crate) struct TimePoint {
    year: u32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)] // care the ordering of the variants is important
pub(crate) enum Duration {
    Length(DurationLength),
    Infinity, // always longer than all other Durations
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DurationLength {
    hours: u32,
    minutes: u8
}


////////////////////////////////////////////////////////////////////
///////////////////////////// Time /////////////////////////////////
////////////////////////////////////////////////////////////////////

impl Time {
    pub(crate) fn new(string: &str) -> Time { //"2009-06-15T13:45:00Z" or "2009-4-15T12:1"
        let shortened = string.replace("Z","");
        let splitted: Vec<&str> = shortened.split(&['T','-',' ',':'][..]).collect();
        let len = splitted.len();
        assert!(len <= 6 && len >= 5, "Wrong time format.");

        let year: u32 = splitted[0].parse().expect("Error at year.");
        let month: u8 = splitted[1].parse().expect("Error at month.");
        assert!(month <= 12 && month >= 1, "Wrong month format.");
        let day: u8 = splitted[2].parse().expect("Error at day.");
        assert!(day <= TimePoint::get_days_of_month(year,month) && day >= 1, "Wrong day format.");
        let hour: u8 = splitted[3].parse().expect("Error at hour.");
        assert!(hour <= 24, "Wrong hour format.");
        let minute: u8 = splitted[4].parse().expect("Error at minute.");
        assert!(minute < 60, "Wrong minute format.");

        Time::Point(TimePoint{
            year,
            month,
            day,
            hour,
            minute})
    }
}

impl Time {
    pub(crate) fn as_iso(&self) -> String {
        match self {
            Time::Earliest => String::from("EARLIEST"),
            Time::Point(t) => format!("{:#04}-{:#02}-{:#02}T{:#02}:{:#02}:00Z",t.year,t.month,t.day,t.hour,t.minute),
            Time::Latest => String::from("LATEST")
        }
    }
}



impl Add<Duration> for Time {
    type Output = Self;

    fn add(self, other: Duration) -> Self {
        match other {
            Duration::Infinity => Time::Latest, //note that Earliest + Infinity = Latest
            Duration::Length(_) => {
                match self {
                    Time::Earliest => Time::Earliest,
                    Time::Point(t) => Time::Point(t + other),
                    Time::Latest => Time::Latest
                }
            }
        }
    }
}


impl Sub for Time {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        assert!(other <= self, "Cannot subtract {} from {}, as it is a later point in time (no negative durations allowed)", other, self);
        match self {
            Time::Earliest => {
                Duration::Length(DurationLength{hours: 0, minutes: 0}) // Earliest - Earliest
            }
            Time::Latest => {
                if other == Time::Latest {
                    Duration::Length(DurationLength{hours: 0, minutes: 0}) // Latest - Latest
                } else {
                    Duration::Infinity // Latest - (something not Latest)
                }

            }
            Time::Point(l1) => {
                match other {
                    Time::Earliest => Duration::Infinity, // Length - Earliest
                    Time::Point(l2) => l1 - l2,
                    _ => panic!("This can never be reached")
                }
            }
        }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Time::Earliest => write!(f, "Earliest"),
            Time::Point(t) => write!(f, "{}", t),
            Time::Latest => write!(f, "Latest")
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
            new_month = new_month - 12;
        }

        TimePoint::correct_date(new_year, new_month, new_day)

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

impl Sub for TimePoint {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        assert!(other <= self, "Cannot subtract {} from {}, as it is a later point in time (no negative durations allowed)", other, self);

        let mut hours = (if self.hour >= other.hour {self.hour - other.hour} else {self.hour + 24 - other.hour}) as u32;
        let minutes = if self.minute >= other.minute {
            self.minute - other.minute
        } else {
            hours = if hours > 0 {hours - 1} else {23}; // subtract one of the hours
            self.minute + 60 - other.minute};


        let mut temp_date = other + Duration::Length(DurationLength{hours, minutes});
        while self != temp_date {
            let days_diff = if self.day > temp_date.day {self.day - temp_date.day} else {self.day + TimePoint::get_days_of_month(temp_date.year, temp_date.month) - temp_date.day};
            let hours_diff = 24 * days_diff as u32;
            temp_date = temp_date + Duration::Length(DurationLength{hours: hours_diff, minutes: 0});
            hours += hours_diff;
        }

        Duration::Length(DurationLength{
            hours,
            minutes
        })
    }
}




impl Add<Duration> for TimePoint {
    type Output = Self;

    fn add(self, other: Duration) -> Self {
        match other {
            Duration::Infinity => {panic!("Infinity should never added to a TimePoint!");},
            Duration::Length(l) => {
                let sum_of_minutes = self.minute + l.minutes;
                let minute = sum_of_minutes % 60;
                let sum_of_hours: u32 = self.hour as u32 + l.hours + (sum_of_minutes/60) as u32;
                let hour = (sum_of_hours % 24) as u8;
                let sum_of_days: u32 = self.day as u32 + sum_of_hours/24;

                let (year,month,day) = TimePoint::correct_date(self.year, self.month, sum_of_days);

                TimePoint{
                    year,
                    month,
                    day,
                    hour,
                    minute
                }
            }
        }
    }
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}.{:02}.{}_{:02}:{:02}", self.day, self.month, self.year, self.hour, self.minute)
    }
}

////////////////////////////////////////////////////////////////////
////////////////////////// Duration ////////////////////////////////
////////////////////////////////////////////////////////////////////

impl Duration {
    pub(crate) fn new(string: &str) -> Duration { // "hh:mm:
        let splitted: Vec<&str> = string.split(&[':'][..]).collect();
        assert!(splitted.len() == 2, "Wrong duration format! string: {}", string);

        let hours: u32 = splitted[0].parse().expect("Error at hour.");
        let minutes: u8 = splitted[1].parse().expect("Error at minute.");
        assert!(minutes < 60, "Wrong minute format.");

        Duration::Length(DurationLength{
            hours,
            minutes
        })
    }

    pub(crate) fn from_iso(string: &str) -> Duration { //"P10DT0H31M0S"
        let splitted: Vec<&str> = string.split(&['P','D','T','H','M','S'][..]).collect();
        assert!(splitted.len() == 7, "Wrong duration format! string: {}", string);

        let days: u32 = splitted[1].parse().expect("Error at days.");
        let hours: u32 = splitted[3].parse().expect("Error at hour.");
        let minutes: u8 = splitted[4].parse().expect("Error at minute.");
        assert!(minutes < 60, "Wrong minute format.");

        Duration::Length(DurationLength{
            hours: hours + 24 * days,
            minutes
        })
    }

    pub(crate) fn zero() -> Duration {
        Duration::Length(DurationLength{
            hours:0,
            minutes:0
        })
    }

}


impl Add for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Duration::Infinity => Duration::Infinity,
            Duration::Length(l1) => {
                match other {
                    Duration::Infinity => Duration::Infinity,
                    Duration::Length(l2) => Duration::Length(l1 + l2)
                }
            }
        }
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert!(self > other, "Cannot subtract a longer duration from a shorter duration.");
        match self {
            Duration::Infinity => Duration::Infinity,
            Duration::Length(l1) => {
                match other {
                    Duration::Infinity => panic!("Cannot subtract Infinity"),
                    Duration::Length(l2) => Duration::Length(l1 - l2)
                }
            }
        }
    }
}

impl std::iter::Sum<Self> for Duration {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Duration::zero(), |a, b| a + b)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Duration::Length(l) => write!(f, "{:02}:{:02}h", l.hours, l.minutes),
            Duration::Infinity => write!(f, "Inf")
        }
    }
}

////////////////////////////////////////////////////////////////////
/////////////////////// DurationLength /////////////////////////////
////////////////////////////////////////////////////////////////////

impl Add for DurationLength {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let sum_of_minutes = self.minutes + other.minutes;
        let minutes = sum_of_minutes % 60;
        let hours = self.hours + other.hours + (sum_of_minutes/60) as u32;
        DurationLength{
            hours,
            minutes
        }
    }
}

impl Sub for DurationLength {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert!(self > other, "Cannot subtract a longer duration from a shorter duration.");
        let mut self_minutes = self.minutes;
        let mut self_hours = self.hours;
        if self.minutes < other.minutes {
            self_minutes += 60;
            self_hours -= 1;
        }
        let minutes = self_minutes - other.minutes;
        let hours = self_hours - other.hours;
        DurationLength{
            hours,
            minutes
        }
    }
}





////////////////////////////////////////////////////////////////////
//////////////////////////// Tests /////////////////////////////////
////////////////////////////////////////////////////////////////////


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
        {
            let earliest = Time::Earliest;
            let dur = Duration::new("50:00");
            assert!(earliest + dur == Time::Earliest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Earliest", earliest, dur, earliest + dur);
        }
        {
            let latest = Time::Latest;
            let dur = Duration::new("50:00");
            assert!(latest + dur == Time::Latest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Latest", latest, dur, latest + dur);
        }
    }
    #[test]
    fn add_infinity_to_time() {
        {
            let time = Time::new("1-01-01T00:00");
            let dur = Duration::Infinity;
            assert!(time + dur == Time::Latest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Latest", time, dur, time + dur);
        }
        {
            let earliest = Time::Earliest;
            let dur = Duration::Infinity;
            assert!(earliest + dur == Time::Latest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Earliest", earliest, dur, earliest + dur);
        }
    }

    #[test]
    fn test_difference_of_two_times() {
        {
            let earlier = Time::new("2022-02-06T16:32");
            let later = Time::new("2022-02-06T16:32");
            let duration = Duration::new("0:00");
            assert!(later - earlier == duration, "Subtracting {} from {} gives {} but should give {}", earlier, later, later - earlier, duration);
            assert!(earlier + (later - earlier) == later, "Adding (later - earlier) to earlier should give later; earlier: {}, later: {}", earlier, later);
        }
        {
            let earlier = Time::new("2022-02-06T16:32");
            let later = Time::new("2022-02-06T17:31");
            let duration = Duration::new("0:59");
            assert!(later - earlier == duration, "Subtracting {} from {} gives {} but should give {}", earlier, later, later - earlier, duration);
            assert!(earlier + (later - earlier) == later, "Adding (later - earlier) to earlier should give later; earlier: {}, later: {}", earlier, later);
        }
        {
            let earlier = Time::new("1989-10-01T02:25");
            let later = Time::new("2022-02-06T17:31");
            let duration = Duration::new("283599:06");
            assert!(later - earlier == duration, "Subtracting {} from {} gives {} but should give {}", earlier, later, later - earlier, duration);
            assert!(earlier + (later - earlier) == later, "Adding (later - earlier) to earlier should give later; earlier: {}, later: {}", earlier, later);
        }
    }

    #[test]
    fn test_difference_of_latest_and_earliest() {
        {
            let earliest = Time::Earliest;
            let later = Time::new("2022-02-06T17:31");
            let duration = Duration::Infinity;
            assert!(later - earliest == duration, "Subtracting {} from {} gives {} but should give {}", earliest, later, later - earliest, duration);
        }
        {
            let earlier = Time::new("2022-02-06T16:32");
            let latest = Time::Latest;
            let duration = Duration::Infinity;
            assert!(latest - earlier == duration, "Subtracting {} from {} gives {} but should give {}", earlier, latest, latest - earlier, duration);
        }
        {
            let earliest = Time::Earliest;
            let latest = Time::Latest;
            let duration = Duration::Infinity;
            assert!(latest - earliest == duration, "Subtracting {} from {} gives {} but should give {}", earliest, latest, latest - earliest, duration);
            assert!(earliest + (latest - earliest) == latest, "Adding (later - earlier) to earlier should give later; earlier: {}, later: {}", earliest, latest);
        }
    }
}
