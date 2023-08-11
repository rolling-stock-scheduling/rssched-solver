use std::fmt;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Sub;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)] // care the ordering of the variants is important
pub enum Duration {
    Length(DurationLength),
    Infinity, // always longer than all other Durations
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct DurationLength {
    pub(super) hours: u32,
    pub(super) minutes: u8,
    pub(super) seconds: u8,
}

////////////////////////////////////////////////////////////////////
////////////////////////// Duration ////////////////////////////////
////////////////////////////////////////////////////////////////////

impl Duration {
    pub fn in_min(&self) -> u32 {
        match self {
            Duration::Infinity => panic!("Cannot get minutes of Duration::Infinity."),
            Duration::Length(l) => l.hours * 60 + (l.minutes as u32),
        }
    }
    pub fn in_sec(&self) -> u32 {
        match self {
            Duration::Infinity => panic!("Cannot get minutes of Duration::Infinity."),
            Duration::Length(l) => l.hours * 3600 + 60 * (l.minutes as u32) + l.seconds as u32,
        }
    }
}

impl Duration {
    pub fn new(string: &str) -> Duration {
        // "hh:mm" or "hh:mm:ss"
        let splitted: Vec<&str> = string.split(&[':'][..]).collect();
        assert!(
            splitted.len() <= 3 && splitted.len() >= 2,
            "Wrong duration format! string: {}",
            string
        );

        let hours: u32 = splitted[0].parse().expect("Error at hour.");
        let minutes: u8 = splitted[1].parse().expect("Error at minute.");
        let seconds: u8 = if splitted.len() == 2 {
            0
        } else {
            splitted[2].parse().expect("Error at second.")
        };
        assert!(minutes < 60, "Wrong minute format.");
        assert!(seconds < 60, "Wrong seconds format.");

        Duration::Length(DurationLength {
            hours,
            minutes,
            seconds,
        })
    }

    pub fn from_seconds(seconds: u32) -> Duration {
        Duration::Length(DurationLength {
            hours: seconds / 3600,
            minutes: (seconds % 3600 / 60) as u8,
            seconds: (seconds % 60) as u8,
        })
    }

    pub fn from_iso(string: &str) -> Duration {
        //"P10DT0H31M02S"
        let splitted: Vec<&str> = string
            .split_inclusive(&['P', 'D', 'T', 'H', 'M', 'S'][..])
            .collect();
        assert!(
            splitted.len() <= 7,
            "Wrong duration format! string: {}",
            string
        );

        let mut days: u32 = 0;
        let mut hours: u32 = 0;
        let mut minutes: u8 = 0;
        let mut seconds: u8 = 0;

        for &s in splitted.iter() {
            match s.chars().last().unwrap() {
                'D' => days = s.replace('D', "").parse().expect("Error at days."),
                'H' => hours = s.replace('H', "").parse().expect("Error at hours."),
                'M' => minutes = s.replace('M', "").parse().expect("Error at minutes."),
                'S' => seconds = s.replace('S', "").parse().expect("Error at seconds."),
                _ => {}
            }
        }

        assert!(minutes < 60, "Wrong minute format.");
        assert!(seconds < 60, "Wrong seconds format.");

        Duration::Length(DurationLength {
            hours: hours + 24 * days,
            minutes,
            seconds,
        })
    }

    pub fn zero() -> Duration {
        Duration::Length(DurationLength {
            hours: 0,
            minutes: 0,
            seconds: 0,
        })
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Duration::Infinity => Duration::Infinity,
            Duration::Length(l1) => match other {
                Duration::Infinity => Duration::Infinity,
                Duration::Length(l2) => Duration::Length(l1 + l2),
            },
        }
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert!(
            self >= other,
            "Cannot subtract a longer duration ({}) from a shorter duration ({}).",
            other,
            self
        );
        match self {
            Duration::Infinity => Duration::Infinity,
            Duration::Length(l1) => match other {
                Duration::Infinity => panic!("Cannot subtract Infinity"),
                Duration::Length(l2) => Duration::Length(l1 - l2),
            },
        }
    }
}

impl Sum for Duration {
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
            Duration::Length(l) => {
                if l.seconds > 0 {
                    write!(f, "{:02}:{:02}:{:02}h", l.hours, l.minutes, l.seconds)
                } else {
                    write!(f, "{:02}:{:02}h", l.hours, l.minutes)
                }
            }
            Duration::Infinity => write!(f, "Inf"),
        }
    }
}

////////////////////////////////////////////////////////////////////
/////////////////////// DurationLength /////////////////////////////
////////////////////////////////////////////////////////////////////

impl Add for DurationLength {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let sum_of_seconds = self.seconds + other.seconds;
        let seconds = sum_of_seconds % 60;
        let sum_of_minutes = self.minutes + other.minutes + (sum_of_seconds / 60) as u8;
        let minutes = sum_of_minutes % 60;
        let hours = self.hours + other.hours + (sum_of_minutes / 60) as u32;
        DurationLength {
            hours,
            minutes,
            seconds,
        }
    }
}

impl Sub for DurationLength {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert!(
            self >= other,
            "Cannot subtract a longer duration from a shorter duration."
        );
        let mut self_seconds = self.seconds;
        let mut self_minutes = self.minutes;
        let mut self_hours = self.hours;
        if self.seconds < other.seconds {
            if self_minutes == 0 {
                self_hours -= 1;
                self_minutes += 60;
            }
            self_minutes -= 1;
            self_seconds += 60;
        }
        if self.minutes < other.minutes {
            self_minutes += 60;
            self_hours -= 1;
        }
        let seconds = self_seconds - other.seconds;
        let minutes = self_minutes - other.minutes;
        let hours = self_hours - other.hours;
        DurationLength {
            hours,
            minutes,
            seconds,
        }
    }
}
