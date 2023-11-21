#[cfg(test)]
use super::*;

#[test]
fn sum_up_duration() {
    let dur1 = Duration::new("5000:40:31");
    let dur2 = Duration::new("00:46:30");
    let sum = Duration::new("5001:27:01");
    assert!(
        dur1 + dur2 == sum,
        "Duration does not sum up correctly. dur1: {} + dur2: {} is {}; but should be {}",
        dur1,
        dur2,
        dur1 + dur2,
        sum
    );
}

#[test]
fn add_duration_to_time_no_leap_year() {
    let time = DateTime::new("1999-2-28T23:40:59");
    let dur = Duration::new("48:46:01");
    let sum = DateTime::new("1999-3-3T00:27");
    assert!(
        time + dur == sum,
        "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be {}",
        time,
        dur,
        time + dur,
        sum
    );
}

#[test]
fn add_duration_to_time_leap_year() {
    let time = DateTime::new("2000-2-28T23:40");
    let dur = Duration::new("48:46:03");
    let sum = DateTime::new("2000-3-2T00:26:03");
    assert!(
        time + dur == sum,
        "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be {}",
        time,
        dur,
        time + dur,
        sum
    );
}

#[test]
fn add_long_duration_to_time() {
    let time = DateTime::new("1-01-01T00:00"); // jesus just got one year old ;)
    let dur = Duration::new("10000000:00:00");
    let sum = DateTime::new("1141-10-18T16:00");
    assert!(
        time + dur == sum,
        "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be {}",
        time,
        dur,
        time + dur,
        sum
    );
}
#[test]
fn add_duration_to_earliest_latest() {
    {
        let earliest = DateTime::Earliest;
        let dur = Duration::new("50:00");
        assert!(earliest + dur == DateTime::Earliest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Earliest", earliest, dur, earliest + dur);
    }
    {
        let latest = DateTime::Latest;
        let dur = Duration::new("50:00");
        assert!(latest + dur == DateTime::Latest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Latest", latest, dur, latest + dur);
    }
}
#[test]
fn add_infinity_to_time() {
    {
        let time = DateTime::new("1-01-01T00:00");
        let dur = Duration::Infinity;
        assert!(time + dur == DateTime::Latest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Latest", time, dur, time + dur);
    }
    {
        let earliest = DateTime::Earliest;
        let dur = Duration::Infinity;
        assert!(earliest + dur == DateTime::Latest, "Duration does not sum up correctly. time: {} + dur: {} is {}; but should be Time::Earliest", earliest, dur, earliest + dur);
    }
}

#[test]
fn test_difference_of_two_times() {
    {
        let earlier = DateTime::new("2022-02-06T16:32:45");
        let later = DateTime::new("2022-02-06T16:32:45");
        let duration = Duration::new("0:00:00");
        assert!(
            later - earlier == duration,
            "Subtracting {} from {} gives {} but should give {}",
            earlier,
            later,
            later - earlier,
            duration
        );
        assert!(
            earlier + (later - earlier) == later,
            "Adding (later - earlier) to earlier should give later; earlier: {}, later: {}",
            earlier,
            later
        );
    }
    {
        let earlier = DateTime::new("2022-02-06T16:32:45");
        let later = DateTime::new("2022-02-06T17:32:44");
        let duration = Duration::new("0:59:59");
        assert!(
            later - earlier == duration,
            "Subtracting {} from {} gives {} but should give {}",
            earlier,
            later,
            later - earlier,
            duration
        );
        assert!(
            earlier + (later - earlier) == later,
            "Adding (later - earlier) to earlier should give later; earlier: {}, later: {}",
            earlier,
            later
        );
    }
    {
        let earlier = DateTime::new("1989-10-01T02:25");
        let later = DateTime::new("2022-02-06T17:31");
        let duration = Duration::new("283599:06:00");
        assert!(
            later - earlier == duration,
            "Subtracting {} from {} gives {} but should give {}",
            earlier,
            later,
            later - earlier,
            duration
        );
        assert!(
            earlier + (later - earlier) == later,
            "Adding (later - earlier) to earlier should give later; earlier: {}, later: {}",
            earlier,
            later
        );
    }
    {
        let earlier = DateTime::new("2000-01-01T23:59:59");
        let later = DateTime::new("2000-01-02T00:00:00");
        let duration = Duration::new("0:00:01");
        assert!(
            later - earlier == duration,
            "Subtracting {} from {} gives {} but should give {}",
            earlier,
            later,
            later - earlier,
            duration
        );
        assert!(
            earlier + (later - earlier) == later,
            "Adding (later - earlier) to earlier should give later; earlier: {}, later: {}",
            earlier,
            later
        );
    }
}

#[test]
fn test_difference_of_latest_and_earliest() {
    {
        let earliest = DateTime::Earliest;
        let later = DateTime::new("2022-02-06T17:31");
        let duration = Duration::Infinity;
        assert!(
            later - earliest == duration,
            "Subtracting {} from {} gives {} but should give {}",
            earliest,
            later,
            later - earliest,
            duration
        );
    }
    {
        let earlier = DateTime::new("2022-02-06T16:32");
        let latest = DateTime::Latest;
        let duration = Duration::Infinity;
        assert!(
            latest - earlier == duration,
            "Subtracting {} from {} gives {} but should give {}",
            earlier,
            latest,
            latest - earlier,
            duration
        );
    }
    {
        let earliest = DateTime::Earliest;
        let latest = DateTime::Latest;
        let duration = Duration::Infinity;
        assert!(
            latest - earliest == duration,
            "Subtracting {} from {} gives {} but should give {}",
            earliest,
            latest,
            latest - earliest,
            duration
        );
        assert!(
            earliest + (latest - earliest) == latest,
            "Adding (later - earlier) to earlier should give later; earlier: {}, later: {}",
            earliest,
            latest
        );
    }
}

#[test]
fn test_subtracting_duration_from_time() {
    {
        let later = DateTime::new("2022-02-06T16:32");
        let duration = Duration::new("0:00:00");
        let earlier = DateTime::new("2022-02-06T16:32");
        assert!(
            later - duration == earlier,
            "Subtracting {} from {} gives {} but should give {}",
            duration,
            later,
            later - duration,
            earlier
        );
        assert!(
            later - (later - earlier) == earlier,
            "Subtracting (later - earlier) from later should give earlier; earlier: {}, later: {}",
            earlier,
            later
        );
    }
    {
        let later = DateTime::new("2022-02-06T17:31:10");
        let duration = Duration::new("0:59:59");
        let earlier = DateTime::new("2022-02-06T16:31:11");
        assert!(
            later - duration == earlier,
            "Subtracting {} from {} gives {} but should give {}",
            duration,
            later,
            later - duration,
            earlier
        );
        assert!(
            later - (later - earlier) == earlier,
            "Subtracting (later - earlier) from later should give earlier; earlier: {}, later: {}",
            earlier,
            later
        );
    }
    {
        let later = DateTime::new("2022-02-06T17:31:00");
        let duration = Duration::new("283599:06:01");
        let earlier = DateTime::new("1989-10-01T02:24:59");
        assert!(
            later - duration == earlier,
            "Subtracting {} from {} gives {} but should give {}",
            duration,
            later,
            later - duration,
            earlier,
        );
        assert!(
            later - (later - earlier) == earlier,
            "Subtracting (later - earlier) from later should give earlier; earlier: {}, later: {}",
            earlier,
            later
        );
    }
}
