use itertools::assert_equal;
use model::base_types::Distance;
use time::{DateTime, Duration};

use crate::{
    path::Path,
    segment::Segment,
    test_utilities::{init_test_data, TestData},
};

use super::Tour;

fn default_tour(d: &TestData) -> Tour {
    Tour::new(
        vec![
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot2,
        ],
        d.network.clone(),
    )
    .unwrap()
}

fn default_path(d: &TestData) -> Path {
    Path::new(vec![d.trip31, d.trip14], d.network.clone())
        .unwrap()
        .unwrap()
}

#[test]
fn basic_methods_test() {
    // ARRANGE
    let d = init_test_data();
    let thirty_minutes = Duration::new("0:30");

    // ACT
    let tour = default_tour(&d);
    let path = default_path(&d);
    let dummy_tour = Tour::new_dummy(path, d.network.clone());

    // ASSERT
    assert!(!tour.is_dummy());
    assert_eq!(tour.nodes.len(), 7);
    assert_eq!(tour.all_non_depot_nodes_iter().count(), 5);
    assert_eq!(tour.useful_duration(), Duration::new("2:30"));
    assert_eq!(tour.service_distance(), Distance::from_meter(15000));
    assert_eq!(tour.dead_head_distance(), Distance::from_meter(12000));
    assert_eq!(
        tour.preceding_overhead(d.start_depot1),
        Ok(Duration::Infinity)
    );
    assert_eq!(
        tour.subsequent_overhead(d.start_depot1),
        Ok(Duration::Infinity)
    );
    assert_eq!(tour.preceding_overhead(d.trip12), Ok(Duration::Infinity));
    assert_eq!(
        tour.subsequent_overhead(d.trip12),
        Ok(Duration::new("0:30"))
    );
    assert_eq!(tour.preceding_overhead(d.trip23), Ok(thirty_minutes));
    assert_eq!(
        tour.subsequent_overhead(d.trip23),
        Ok(Duration::new("0:30"))
    );
    assert_eq!(tour.preceding_overhead(d.trip34), Ok(thirty_minutes));
    assert_eq!(
        tour.subsequent_overhead(d.trip34),
        Ok(Duration::new("0:30"))
    );
    assert_eq!(tour.preceding_overhead(d.trip45), Ok(thirty_minutes));
    assert_eq!(
        tour.subsequent_overhead(d.trip45),
        Ok(Duration::new("0:30"))
    );
    assert_eq!(tour.preceding_overhead(d.trip51), Ok(thirty_minutes));
    assert_eq!(tour.subsequent_overhead(d.trip51), Ok(Duration::Infinity));
    assert_eq!(
        tour.preceding_overhead(d.end_depot2),
        Ok(Duration::Infinity)
    );
    assert_eq!(
        tour.subsequent_overhead(d.end_depot2),
        Ok(Duration::Infinity)
    );
    assert!(tour.preceding_overhead(d.trip31).is_err());
    assert!(tour.subsequent_overhead(d.trip31).is_err());
    assert_eq!(tour.first_node(), d.start_depot1);
    assert_eq!(tour.last_node(), d.end_depot2);
    assert_eq!(tour.nth_node(0), Some(d.start_depot1));
    assert_eq!(tour.nth_node(1), Some(d.trip12));
    assert_eq!(tour.nth_node(2), Some(d.trip23));
    assert_eq!(tour.nth_node(3), Some(d.trip34));
    assert_eq!(tour.nth_node(4), Some(d.trip45));
    assert_eq!(tour.nth_node(5), Some(d.trip51));
    assert_eq!(tour.nth_node(6), Some(d.end_depot2));
    assert_eq!(tour.nth_node(7), None);
    assert_eq!(tour.start_time(), DateTime::new("2020-01-01T06:00"));
    assert_eq!(tour.end_time(), DateTime::new("2020-01-01T11:15"));
    assert_eq!(tour.latest_not_reaching_node(d.trip31), Some(3)); // trip34 on pos 3
    assert_eq!(tour.latest_not_reaching_node(d.end_depot1), Some(6)); // every node can reach
                                                                      // end_depot, except for the
                                                                      // end_depot2
    assert_eq!(tour.latest_not_reaching_node(d.start_depot1), Some(0)); // start_depot1 cannot
                                                                        // reach start_depot1

    assert!(dummy_tour.is_dummy());
    assert_eq!(dummy_tour.nodes.len(), 2);
    assert_eq!(dummy_tour.all_non_depot_nodes_iter().count(), 2);
    assert_eq!(dummy_tour.useful_duration(), Duration::new("1:00"));
    assert_eq!(dummy_tour.service_distance(), Distance::from_meter(13000));
    assert_eq!(dummy_tour.dead_head_distance(), Distance::zero());
    assert_eq!(
        dummy_tour.preceding_overhead(d.trip31),
        Ok(Duration::Infinity)
    );
    assert_eq!(dummy_tour.subsequent_overhead(d.trip31), Ok(thirty_minutes));
    assert_eq!(dummy_tour.preceding_overhead(d.trip14), Ok(thirty_minutes));
    assert_eq!(
        dummy_tour.subsequent_overhead(d.trip14),
        Ok(Duration::Infinity)
    );
    assert_eq!(dummy_tour.first_node(), d.trip31);
    assert_eq!(dummy_tour.last_node(), d.trip14);
    assert_eq!(dummy_tour.nth_node(0), Some(d.trip31));
    assert_eq!(dummy_tour.nth_node(1), Some(d.trip14));
    assert_eq!(dummy_tour.nth_node(2), None);
    assert_eq!(dummy_tour.start_time(), DateTime::new("2020-01-01T08:00"));
    assert_eq!(dummy_tour.end_time(), DateTime::new("2020-01-01T09:30"));
}

#[test]
fn sub_path_tests() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);

    // ACT
    let sub_path1 = tour.sub_path(Segment::new(d.trip12, d.trip45)).unwrap();
    let sub_path2 = tour
        .sub_path(Segment::new(d.start_depot1, d.trip23))
        .unwrap();
    let sub_path3 = tour.sub_path(Segment::new(d.trip23, d.end_depot2)).unwrap();
    let sub_path4 = tour
        .sub_path(Segment::new(d.start_depot1, d.end_depot2))
        .unwrap();
    let result5 = tour.sub_path(Segment::new(d.trip31, d.trip51));

    // ASSERT
    assert_equal(
        sub_path1.iter(),
        [d.trip12, d.trip23, d.trip34, d.trip45].iter().cloned(),
    );

    assert_equal(
        sub_path2.iter(),
        [d.start_depot1, d.trip12, d.trip23].iter().cloned(),
    );

    assert_equal(
        sub_path3.iter(),
        [d.trip23, d.trip34, d.trip45, d.trip51, d.end_depot2]
            .iter()
            .cloned(),
    );

    assert_equal(
        sub_path4.iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );

    assert!(result5.is_err());
}

#[test]
fn invalid_constructor_test() {
    // ARRANGE
    let d = init_test_data();

    // ACT
    let invalid_tour1 = Tour::new(vec![d.start_depot1, d.end_depot1], d.network.clone());
    let invalid_tour2 = Tour::new(
        vec![d.end_depot1, d.trip12, d.end_depot1],
        d.network.clone(),
    );
    let invalid_tour3 = Tour::new(
        vec![d.start_depot1, d.trip12, d.start_depot1],
        d.network.clone(),
    );
    let invalid_tour4 = Tour::new(
        vec![
            d.start_depot1,
            d.trip12,
            d.start_depot2,
            d.trip23,
            d.end_depot1,
        ],
        d.network.clone(),
    );
    let invalid_tour5 = Tour::new(
        vec![d.start_depot1, d.trip23, d.trip12, d.end_depot2],
        d.network.clone(),
    );
    let invalid_dummy_tour1 = Tour::new(vec![d.start_depot1, d.trip12], d.network.clone());
    let invalid_dummy_tour2 = Tour::new(vec![d.trip12, d.end_depot1], d.network.clone());
    let invalid_dummy_tour3 = Tour::new(vec![d.trip23, d.trip12], d.network.clone());

    // ASSERT
    assert!(invalid_tour1.is_err());
    assert!(invalid_tour2.is_err());
    assert!(invalid_tour3.is_err());
    assert!(invalid_tour4.is_err());
    assert!(invalid_tour5.is_err());
    assert!(invalid_dummy_tour1.is_err());
    assert!(invalid_dummy_tour2.is_err());
    assert!(invalid_dummy_tour3.is_err());
}

#[test]
fn conflict_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let segment = Segment::new(d.trip31, d.trip14);

    // ACT
    let conflicted_path = tour.conflict(segment);

    // ASSERT
    assert_equal(
        conflicted_path.unwrap().iter(),
        [d.trip34, d.trip45, d.trip51].iter().cloned(),
    );
}
#[test]
fn insert_path_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let path = default_path(&d);

    // ACT
    let (new_tour, removed_path_option) = tour.insert_path(path);

    // ASSERT
    assert_equal(
        new_tour.all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip14,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );

    let removed_path = removed_path_option.unwrap();
    assert_equal(
        removed_path.iter(),
        [d.trip34, d.trip45, d.trip51].iter().cloned(),
    );
    assert_eq!(new_tour.dead_head_distance(), Distance::from_meter(42000));
    assert_eq!(new_tour.useful_duration(), Duration::new("2:00"));
    assert_eq!(new_tour.service_distance(), Distance::from_meter(16000));
    assert_eq!(new_tour.start_time(), DateTime::new("2020-01-01T06:00"));
    assert_eq!(new_tour.end_time(), DateTime::new("2020-01-01T10:15"));
}

#[test]
fn replace_start_depot_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let new_start_depot = d.start_depot2;

    // ACT
    let replace_result = tour.replace_start_depot(new_start_depot);

    // ASSERT
    assert!(replace_result.is_ok());
    let new_tour = replace_result.unwrap();
    assert_equal(
        new_tour.all_nodes_iter(),
        [
            d.start_depot2,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
    assert_eq!(new_tour.start_time(), DateTime::new("2020-01-01T05:15"));
    assert_eq!(new_tour.dead_head_distance(), Distance::from_meter(33000));
}

#[test]
fn replace_end_depot_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let new_end_depot = d.end_depot1;

    // ACT
    let replace_result = tour.replace_end_depot(new_end_depot);

    // ASSERT
    assert!(replace_result.is_ok());
    let new_tour = replace_result.unwrap();
    assert_equal(
        new_tour.all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );
    assert_eq!(new_tour.end_time(), DateTime::new("2020-01-01T10:30"));
    assert_eq!(new_tour.dead_head_distance(), Distance::from_meter(0));
}

#[test]
fn removable_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let segment_for_removal = Segment::new(d.trip12, d.trip45);

    // ACT
    let removable_result = tour.check_removable(segment_for_removal);

    // ASSERT
    assert!(removable_result.is_ok());
}

#[test]
fn remove_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let segment_for_removal = Segment::new(d.trip12, d.trip45);

    // ACT
    let remove_result = tour.remove(segment_for_removal);

    // ASSERT
    assert!(remove_result.is_ok());
    let (new_tour_option, removed_path) = remove_result.unwrap();
    let new_tour = new_tour_option.unwrap();
    assert_equal(
        new_tour.all_nodes_iter(),
        [d.start_depot1, d.trip51, d.end_depot2].iter().cloned(),
    );

    assert_equal(
        removed_path.iter(),
        [d.trip12, d.trip23, d.trip34, d.trip45].iter().cloned(),
    );
    assert_eq!(new_tour.dead_head_distance(), Distance::from_meter(27000));
    assert_eq!(new_tour.useful_duration(), Duration::new("0:30"));
    assert_eq!(new_tour.service_distance(), Distance::from_meter(5000));
    assert_eq!(new_tour.start_time(), DateTime::new("2020-01-01T09:15"));
    assert_eq!(new_tour.end_time(), DateTime::new("2020-01-01T11:15"));
}

#[test]
fn sub_path_test() {
    // ARRANGE

    let d = init_test_data();
    let tour = default_tour(&d);
    let segment = Segment::new(d.trip12, d.trip45);

    // ACT
    let sub_path_result = tour.sub_path(segment);

    // ASSERT
    assert!(sub_path_result.is_ok());
    let sub_path = sub_path_result.unwrap();
    assert_equal(
        sub_path.iter(),
        [d.trip12, d.trip23, d.trip34, d.trip45].iter().cloned(),
    );
}

// more complex modification tests
#[test]
fn insert_path_with_start_depot_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let path = Path::new(vec![d.start_depot2, d.trip31], d.network.clone())
        .unwrap()
        .unwrap();

    // ACT
    let (new_tour, removed_path_option) = tour.insert_path(path);

    // ASSERT
    assert_equal(
        new_tour.all_nodes_iter(),
        [d.start_depot2, d.trip31, d.trip51, d.end_depot2]
            .iter()
            .cloned(),
    );
    assert_equal(
        removed_path_option.unwrap().iter(),
        [d.start_depot1, d.trip12, d.trip23, d.trip34, d.trip45]
            .iter()
            .cloned(),
    );
    assert_eq!(
        new_tour.dead_head_distance(),
        Distance::from_meter(23000 + 15000 + 12000)
    );
    assert_eq!(new_tour.useful_duration(), Duration::new("1:00"));
    assert_eq!(new_tour.service_distance(), Distance::from_meter(11000));
    assert_eq!(new_tour.start_time(), DateTime::new("2020-01-01T07:15"));
    assert_eq!(new_tour.end_time(), DateTime::new("2020-01-01T11:15"));
}

#[test]
fn insert_path_with_end_depot_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let path = Path::new(vec![d.trip14, d.end_depot1], d.network.clone())
        .unwrap()
        .unwrap();

    // ACT
    let (new_tour, removed_path_option) = tour.insert_path(path);

    // ASSERT
    assert_equal(
        new_tour.all_nodes_iter(),
        [d.start_depot1, d.trip12, d.trip23, d.trip14, d.end_depot1]
            .iter()
            .cloned(),
    );
    assert_equal(
        removed_path_option.unwrap().iter(),
        [d.trip34, d.trip45, d.trip51, d.end_depot2].iter().cloned(),
    );
    assert_eq!(
        new_tour.dead_head_distance(),
        Distance::from_meter(31000 + 41000)
    );
    assert_eq!(new_tour.useful_duration(), Duration::new("1:30"));
    assert_eq!(new_tour.service_distance(), Distance::from_meter(10000));
    assert_eq!(new_tour.start_time(), DateTime::new("2020-01-01T06:00"));
    assert_eq!(new_tour.end_time(), DateTime::new("2020-01-01T10:15"));
}

#[test]
fn insert_path_with_start_and_end_depot_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let path = Path::new(
        vec![d.start_depot2, d.trip31, d.trip14, d.end_depot1],
        d.network.clone(),
    )
    .unwrap()
    .unwrap();

    // ACT
    let (new_tour, removed_path_option) = tour.insert_path(path);

    // ASSERT
    assert_equal(
        new_tour.all_nodes_iter(),
        [d.start_depot2, d.trip31, d.trip14, d.end_depot1]
            .iter()
            .cloned(),
    );
    assert_equal(
        removed_path_option.unwrap().iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
    assert_eq!(
        new_tour.dead_head_distance(),
        Distance::from_meter(23000 + 41000)
    );
    assert_eq!(new_tour.useful_duration(), Duration::new("1:00"));
    assert_eq!(new_tour.service_distance(), Distance::from_meter(13000));
    assert_eq!(new_tour.start_time(), DateTime::new("2020-01-01T07:15"));
    assert_eq!(new_tour.end_time(), DateTime::new("2020-01-01T10:15"));
}

#[test]
fn insert_path_such_that_only_depot_is_removed_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = Tour::new(
        vec![d.start_depot1, d.trip34, d.trip45, d.end_depot2],
        d.network.clone(),
    )
    .unwrap();
    let path1 = Path::new(vec![d.start_depot2, d.trip12], d.network.clone())
        .unwrap()
        .unwrap();

    let path2 = Path::new(vec![d.trip51, d.end_depot1], d.network.clone())
        .unwrap()
        .unwrap();

    let path3 = Path::new(vec![d.trip12, d.trip23], d.network.clone())
        .unwrap()
        .unwrap();

    // ACT
    let (new_tour1, removed_path_option1) = tour.insert_path(path1);
    let (new_tour2, removed_path_option2) = tour.insert_path(path2);
    let (new_tour3, removed_path_option3) = tour.insert_path(path3);

    // ASSERT
    assert_equal(
        new_tour1.all_nodes_iter(),
        [d.start_depot2, d.trip12, d.trip34, d.trip45, d.end_depot2]
            .iter()
            .cloned(),
    );
    assert!(removed_path_option1.is_none());

    assert_equal(
        new_tour2.all_nodes_iter(),
        [d.start_depot1, d.trip34, d.trip45, d.trip51, d.end_depot1]
            .iter()
            .cloned(),
    );
    assert!(removed_path_option2.is_none());

    assert_equal(
        new_tour3.all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
    assert!(removed_path_option3.is_none());
}

#[test]
fn insert_path_with_depot_to_dummy_tour() {
    // ARRANGE
    let d = init_test_data();
    let path = Path::new(
        vec![d.trip12, d.trip23, d.trip34, d.trip45, d.trip51],
        d.network.clone(),
    )
    .unwrap()
    .unwrap();
    let dummy_tour = Tour::new_dummy(path, d.network.clone());

    let path = Path::new(
        vec![d.start_depot1, d.trip31, d.trip14, d.end_depot2],
        d.network.clone(),
    )
    .unwrap()
    .unwrap();

    // ACT
    let (new_dummy_tour, removed_path_option) = dummy_tour.insert_path(path);

    // ASSERT
    assert_equal(
        new_dummy_tour.all_nodes_iter(),
        [d.trip12, d.trip23, d.trip31, d.trip14].iter().cloned(),
    );
    assert_equal(
        removed_path_option.unwrap().iter(),
        [d.trip34, d.trip45, d.trip51].iter().cloned(),
    );
    assert_eq!(new_dummy_tour.dead_head_distance(), Distance::from_meter(0));
    assert_eq!(new_dummy_tour.useful_duration(), Duration::new("2:00"));
    assert_eq!(
        new_dummy_tour.service_distance(),
        Distance::from_meter(16000)
    );
    assert_eq!(
        new_dummy_tour.start_time(),
        DateTime::new("2020-01-01T06:00")
    );
    assert_eq!(new_dummy_tour.end_time(), DateTime::new("2020-01-01T09:30"));
}

#[test]
fn remove_all_nodes_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let all_inner_nodes_segment = Segment::new(d.trip12, d.trip51);
    let start_depot_segment = Segment::new(d.start_depot1, d.trip51);
    let end_depot_segment = Segment::new(d.trip12, d.end_depot2);
    let all_nodes_segment = Segment::new(d.start_depot1, d.end_depot2);

    // ACT
    let all_inner_nodes_result = tour.remove(all_inner_nodes_segment);
    let start_depot_result = tour.remove(start_depot_segment);
    let end_depot_result = tour.remove(end_depot_segment);
    let all_nodes_result = tour.remove(all_nodes_segment);

    // ASSERT
    assert!(all_inner_nodes_result.is_ok());
    let (new_tour_option, removed_path) = all_inner_nodes_result.unwrap();
    assert!(new_tour_option.is_none());
    assert_equal(
        removed_path.iter(),
        [d.trip12, d.trip23, d.trip34, d.trip45, d.trip51]
            .iter()
            .cloned(),
    );

    assert!(start_depot_result.is_ok());
    let (new_tour_option, removed_path) = start_depot_result.unwrap();
    assert!(new_tour_option.is_none());
    assert_equal(
        removed_path.iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
        ]
        .iter()
        .cloned(),
    );

    assert!(end_depot_result.is_ok());
    let (new_tour_option, removed_path) = end_depot_result.unwrap();
    assert!(new_tour_option.is_none());
    assert_equal(
        removed_path.iter(),
        [
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );

    assert!(all_nodes_result.is_ok());
    let (new_tour_option, removed_path) = all_nodes_result.unwrap();
    assert!(new_tour_option.is_none());
    assert_equal(
        removed_path.iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
}

#[test]
fn remove_single_node_test() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let segment_for_removal = Segment::new(d.trip12, d.trip12);

    // ACT
    let remove_result = tour.remove(segment_for_removal);

    // ASSERT
    assert!(remove_result.is_ok());
    let (new_tour_option, removed_path) = remove_result.unwrap();
    assert_equal(
        new_tour_option.unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
    assert_equal(removed_path.iter(), [d.trip12].iter().cloned());
}

#[test]
fn remove_invalid_segment() {
    // ARRANGE
    let d = init_test_data();
    let tour = default_tour(&d);
    let reversed_segment = Segment::new(d.trip23, d.trip12);
    let start_invalid_segment = Segment::new(d.trip31, d.trip45);
    let end_invalid_segment = Segment::new(d.trip12, d.trip14);
    let start_depot_segment = Segment::new(d.start_depot1, d.start_depot1);
    let end_depot_segment = Segment::new(d.end_depot2, d.end_depot2);

    // ACT
    let reversed_result = tour.remove(reversed_segment);
    let start_invalid_result = tour.remove(start_invalid_segment);
    let end_invalid_result = tour.remove(end_invalid_segment);
    let start_depot_result = tour.remove(start_depot_segment);
    let end_depot_result = tour.remove(end_depot_segment);

    // ASSERT
    assert!(reversed_result.is_err());
    assert!(start_invalid_result.is_err());
    assert!(end_invalid_result.is_err());
    assert!(start_depot_result.is_err());
    assert!(end_depot_result.is_err());
}
