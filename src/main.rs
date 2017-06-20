#[macro_use(mem_info)]
extern crate arrayfire as af;

use std::collections::HashMap;
use std::time::Instant;
use af::*;

const NUM_CANDIDATES: u64 = 3;
const ADVERSARIES: u64 = 2;
const MIDDLE_SAMPLES: u64 = 21;
const SCENARIOS: u64 = 500_000;

const ERROR: u64 = 1;
const WARN: u64 = 2;
const INFO: u64 = 3;
const DEBUG: u64 = 4;
const TRACE: u64 = 5;
const LOGLEVEL: u64 = 3;

#[allow(unused_must_use)]
#[allow(unused_variables)]
#[allow(dead_code)]
fn main() {
    println!("Starting...");
    set_device(0);
    info();
    let dims = Dim4::new(&[SCENARIOS, 1, 1, 1]);
    let xcand = Dim4::new(&[1, NUM_CANDIDATES, 1, 1]);

// nanos for various runs

    //204629241 1 sample points,20_000_000 scenarios
    // 16345490 1 sample points,   500_000 scenarios
    // 21556012 1 sample points, 1_000_000 scenarios

    // 22111685 6 sample points,   500_000 scenarios
    // 29723635 6 sample points, 1_000_000 scenarios
    // 59659298 5^2 unmemozied,500_000
    // 64700557 5^2 memoized ballots....really?
    // 65490278 5^2 unmemozied,500_000 --release...what?
    //15011194632

    //222172629 10^2 win_points^2 (not pretab), 500_000 sc
    // with win_points pretabulated:
    // 73593116 10 sample,1_000_000 scenarios
    // 57679105 10 sample,  500_000 scenarios
    // 46587156 10 sample,  100_000 scenarios
    // 38447700  6 sample,  500_000 scenarios

    let start = Instant::now();

    mem_info!("Before benchmark");
    let utility_matrix = {
        let middle_points = (0..MIDDLE_SAMPLES).map(|x| x as f64/((MIDDLE_SAMPLES-1) as f64)).collect::<Vec<_>>();
        /* TODO parameter num adversaries */
        let normalized_a1a2 = &get_adversary_ballots_combined(); if LOGLEVEL >= DEBUG {af_print!("a1 + a2 normalized:", normalized_a1a2);}

        // pretabulate and store win_counts for each middle point so we only calc once per each
        let mut win_counts : Vec<Array> = Vec::new();
        for i in 0..middle_points.len() {
            let &middle_point = &middle_points[i];
            win_counts.push(get_win_count_using_middle(middle_point, normalized_a1a2))
        }

        let mut utility_matrix = [[0.0; MIDDLE_SAMPLES as usize]; MIDDLE_SAMPLES as usize];
        for i in 0..middle_points.len() {
            for j in 0..middle_points.len() {
                let &honest_point = &middle_points[i];
                let &middle_point = &middle_points[j];

                // protagonist's info
                let win_count = &win_counts[j];
                if LOGLEVEL >= TRACE { af_print!("win_count: ", win_count); }
                let utility_total = get_utility_total(win_count, &Array::new(&[0.0, honest_point, 1.0], Dim4::new(&[1, NUM_CANDIDATES, 1, 1])));
                let utility_percent = utility_total as f64 / (SCENARIOS as f64);
                utility_matrix[i][j] = utility_percent;
                println!("Honest middle: {}|Percent of possible utility for {} strat: {}", honest_point, middle_point, utility_percent);
            }
        }
        utility_matrix
    };

    println!("utility matrix: {:?}", utility_matrix);

    println!("found in: {:?}", start.elapsed());
    mem_info!("After benchmark");
}

fn get_utility_total(win_counts: &Array, utilities: &Array) -> f64 {
    let x = &mul(win_counts, utilities, false);
    if LOGLEVEL >= TRACE { af_print!("x: ", x); }
    let (util, imag) = sum_all(x);
    assert!(imag == 0.0);
    util
}

/// protagonist will vote using (0, middle_point, 1) in each scenario, then we return an array which tallies the wins of each candidate.
/// near_total represents the ballot totals minus the protagonists ballot, all that remains is to add in the normalized ballot of protagonist.
fn get_win_count_using_middle(middle_point: f64, near_total: &Array) -> Array {
    let my_ballot = &Array::new(&[0.0, middle_point, 1.0], Dim4::new(&[1, NUM_CANDIDATES, 1, 1]));
    if LOGLEVEL >= TRACE { af_print!("honest: ", my_ballot); }
    //calculate utility total of honest strat
    get_win_count(my_ballot, near_total)
}

fn get_win_count(utilities: &Array, near_total: &Array) -> Array {
    let normalized_honest = &normalize_ballot(utilities);
    if LOGLEVEL >= TRACE { af_print!("honest normalized: ", normalized_honest); }
    let result = &add(near_total, normalized_honest, true);
    if LOGLEVEL >= TRACE { af_print!("result: ", result); }
    let winners = &(imax(result, 1).1);
    if LOGLEVEL >= TRACE { af_print!("winners: ", winners); }
    transpose(&histogram(winners, 3, 0.0, 2.0), false)
}

fn get_adversary_ballots_combined() -> Array {
    let dimsxcand = Dim4::new(&[SCENARIOS, NUM_CANDIDATES, 1, 1]);
    // first adversary's votes
    let a1 = &randu::<f32>(dimsxcand);
    //af_print!("Just initialized a1:", a1);
    let normalized_a1 = &normalize_ballot(a1);
    //af_print!("a1 normalized:", normalized_a1);
    // second adversary's votes
    let a2 = &randu::<f32>(dimsxcand);
    //af_print!("Just initialized a2:", a2);
    let normalized_a2 = &normalize_ballot(a2);
    //af_print!("a2 normalized:", normalized_a2);
    add(normalized_a1, normalized_a2, false)
}

/// simple stddev normalize
//fn normalize_ballot(input: &Array) -> Array {
//    let a1_stdev = &stdev(input, 1);
//    //af_print!("a1_stdev:", a1_stdev);
//    mem_info!("end normalize");
//    div(input, a1_stdev, true)
//}

/// this normalize will be dynamic based on parameters
fn normalize_ballot(input: &Array) -> Array {
    let mean1 = &mean(input, 1);
    println!("1");
    //af_print!("mean:", mean1);
    let subbed = &sub(input, mean1, true);
    println!("2");
    let powed = &pow(subbed, &Array::new(&[2.0], Dim4::new(&[1, 1, 1, 1])), true);
    println!("3");
    let mean2 = &mean(powed, 1);
    println!("4");
    let retval = pow(mean2, &Array::new(&[1.0/2.0], Dim4::new(&[1, 1, 1, 1])), true);
    mem_info!("end normalize");
    retval
}