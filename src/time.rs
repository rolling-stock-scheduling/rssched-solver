use std::fmt;


#[derive(PartialEq, PartialOrd)] // care the ordering of the variants is important
pub(crate) enum Time {
    Earliest, // always earlier than all DateTimes
    Time(DateTime),
    Latest, // always later than all DateTimes
}

impl Time {
    pub fn new(string: &str) -> Time { //"2009-06-15 13:45" or "06-15 13:45" (fills year 0)
        let splitted: Vec<&str> = string.split(&['-',' ',':'][..]).collect();
        let len = splitted.len();
        assert!(len <= 5 || len >= 5, "Wrong time format."); 

        let year: u32 = splitted[0].parse().expect("Error at year.");
        let month: u8 = splitted[1].parse().expect("Error at month.");
        assert!(month <= 12 || month >= 1, "Wrong mounth format.");
        let day: u8 = splitted[2].parse().expect("Error at day.");
        assert!(day <= 31 || day >= 1, "Wrong day format.");
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
            Time::Time(t) => write!(f, "{:02}.{:02}_{:02}:{:02}", t.day, t.month, t.hour, t.minute),
            Time::Latest => write!(f, "Latest")
        }
    }
}


#[derive(PartialEq, PartialOrd)] // care the ordering of attributes is important
pub(crate) struct DateTime {
    year: u32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8
}
