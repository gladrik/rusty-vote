#[macro_use(mem_info)]
#[allow(unused_must_use)]
#[allow(unused_variables)]
#[allow(unused_constants)]
#[allow(unused_imports)]
#[allow(dead_code)]
extern crate arrayfire as af;

use std::thread::sleep;
use std::time::Instant;
use std::time::Duration;
use af::*;

const NUM_CANDIDATES: u64 = 3;
const ADVERSARIES: u64 = 2;
const MIDDLE_SAMPLES: u64 = 11;
const SCENARIOS: u64 = 5_000_000;

const ERROR: u64 = 1;
const WARN: u64 = 2;
const INFO: u64 = 3;
const DEBUG: u64 = 4;
const TRACE: u64 = 5;
const LOGLEVEL: u64 = INFO;


fn main() {
    println!("Starting...");
    set_device(0);
    info();

    let start = Instant::now();

    mem_info!("Before benchmark");
    let utility_matrix = {
        let middle_points = (0..MIDDLE_SAMPLES).map(|x| x as f64 / ((MIDDLE_SAMPLES - 1) as f64)).collect::<Vec<_>>();
        /* TODO allow adjustable number of adversaries (currently 2) */
        // normalized_a1a2 is a matrix with the combined scores of all adversaries, with a row for each scenario
        let normalized_a1a2 = &get_adversary_ballots_combined();
        if LOGLEVEL >= DEBUG { af_print!("a1 + a2 normalized:", normalized_a1a2); }

        // precalculate the win_counts for each middle point
        let mut win_counts: Vec<Array<u32>> = Vec::new();
        for i in 0..middle_points.len() {
            let &middle_point = &middle_points[i];
            win_counts.push(get_win_count_using_middle(middle_point, normalized_a1a2))
        }

        // use the win counts and the utilities to determine the performance of middle votes at each middle point
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

fn get_utility_total(win_counts: &Array<u32>, utilities: &Array<f64>) -> f64 {
    let x = &mul(win_counts, utilities, false);
    if LOGLEVEL >= TRACE { af_print!("x: ", x); }
    let (util, imag) = sum_all(x);
    assert_eq!(imag, 0.0);
    util
}

/// protagonist will vote using (0, middle_point, 1) in each scenario, then we return an array which tallies the wins of each candidate.
/// near_total represents the ballot totals minus the protagonists ballot, all that remains is to add in the normalized ballot of protagonist.
fn get_win_count_using_middle(middle_point: f64, near_total: &Array<f64>) -> Array<u32> {
    let my_ballot = &Array::new(&[0.0, middle_point, 1.0], Dim4::new(&[1, NUM_CANDIDATES, 1, 1]));
    if LOGLEVEL >= TRACE { af_print!("honest: ", my_ballot); }
    //calculate utility total of strat
    get_win_count(my_ballot, near_total)
}

fn get_win_count(utilities: &Array<f64>, near_total: &Array<f64>) -> Array<u32> {
    let normalized_honest = &normalize_ballot_maxdev(utilities);
    if LOGLEVEL >= TRACE { af_print!("honest normalized: ", normalized_honest); }
    let result = &add(near_total, normalized_honest, true);
    if LOGLEVEL >= TRACE { af_print!("result: ", result); }
    let winners = &(imax(result, 1).1);
    if LOGLEVEL >= TRACE { af_print!("winners: ", winners); }
    transpose(&histogram(winners, 3, 0.0, 2.0), false)
}

fn get_adversary_ballots_combined() -> Array<f64> {
    let dimsxcand = Dim4::new(&[SCENARIOS, NUM_CANDIDATES, 1, 1]);
    // first adversary's votes
    let a1 = &randu::<f64>(dimsxcand);
    //af_print!("Just initialized a1:", a1);
    let normalized_a1 = &normalize_ballot_maxdev(a1);
    //af_print!("a1 normalized:", normalized_a1);
    // second adversary's votes
    let a2 = &randu::<f64>(dimsxcand);
    //af_print!("Just initialized a2:", a2);
    let normalized_a2 = &normalize_ballot_maxdev(a2);
    //af_print!("a2 normalized:", normalized_a2);
    add(normalized_a1, normalized_a2, false)
}

/// TODO create normalize method which can be dynamic based on parameters
/// this normalize function produces a divisor by using a non-standard deviation:
/// this deviation is like standard deviation but subtracts the maximum from each point rather than the mean
/// input typically has dimensions Dim4::new(&[SCENARIOS, NUM_CANDIDATES, 1, 1]);
fn normalize_ballot_maxdev(input: &Array<f64>) -> Array<f64> {
    // af_print!("input:", input);
    let max = &max(input, 1);
    let maximums = &sub(max, &Array::new(&[0.0], Dim4::new(&[1, 1, 1, 1])), true);
    // af_print!("maximums:", maximums);
    let difference = &sub(input, maximums, true);
    // af_print!("difference:", difference);
    let squared = &pow(difference, &Array::new(&[2.0], Dim4::new(&[1, 1, 1, 1])), true);
    // af_print!("squared:", squared);
    let mean2 = &mean(squared, 1);
    let nonstandard_deviation = &pow(mean2, &Array::new(&[1.0 / 2.0], Dim4::new(&[1, 1, 1, 1])), true);
    let retval = div(input, nonstandard_deviation, true);
    // af_print!("nonstandard_deviation", nonstandard_deviation);

    //mem_info!("end normalize");
    retval
}

/// simple stddev normalize
fn normalize_ballot_standard(input: &Array<f64>) -> Array<f64> {
    let a1_stdev = &stdev(input, 1);
    //af_print!("a1_stdev:", a1_stdev);
    mem_info!("end normalize");
    div(input, a1_stdev, true)
}
