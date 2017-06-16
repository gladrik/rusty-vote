#[macro_use(mem_info)]
extern crate arrayfire as af;

use std::collections::HashMap;
use std::time::Instant;
use af::*;

static NUM_CANDIDATES: u64 = 3;
static SCENARIOS: u64 = 500_000;

static ERROR: u64 = 1;
static WARN: u64 = 2;
static INFO: u64 = 3;
static DEBUG: u64 = 4;
static TRACE: u64 = 5;
static LOGLEVEL: u64 = 3;

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

    let start = Instant::now();

    mem_info!("Before benchmark");
    {
        let middle_points = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        /* TODO parameter num adversaries */
        let normalized_a1a2 = &get_adversary_ballots_combined(); if LOGLEVEL >= DEBUG {af_print!("a1 + a2 normalized:", normalized_a1a2);}

        for i in 0..middle_points.len() {
            for j in 0..middle_points.len() {
                let &honest_point = &middle_points[i];
                let &middle_point = &middle_points[j];

                // protagonist's info
                let win_count = { // TODO float to wincounts function, memoize it
                    let my_ballot = &Array::new(&[0.0, middle_point, 1.0], Dim4::new(&[1, NUM_CANDIDATES, 1, 1]));
                    if LOGLEVEL >= TRACE { af_print!("honest: ", my_ballot); }
                    //calculate utility total of honest strat
                    &get_win_counts(my_ballot, normalized_a1a2)
                };
                if LOGLEVEL >= TRACE { af_print!("win_count: ", win_count); }
                let utility_total = get_utility_total(win_count, &Array::new(&[0.0, honest_point, 1.0], Dim4::new(&[1, NUM_CANDIDATES, 1, 1])));
                println!("Honest middle: {}|Percent of possible utility for {} strat: {}", honest_point, middle_point, utility_total as f64 / (SCENARIOS as f64));
            }
        }

        println!("found in: {:?}", start.elapsed());
    }

    mem_info!("After benchmark");
}

fn get_utility_total(win_counts: &Array, utilities: &Array) -> f64 {
    let x = &mul(win_counts, utilities, false);
    if LOGLEVEL >= TRACE { af_print!("x: ", x); }
    let (util, imag) = sum_all(x);
    assert!(imag == 0.0);
    util
}

fn get_win_counts(utilities: &Array, near_total: &Array) -> Array {
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

fn normalize_ballot(input: &Array) -> Array {
    let a1_stdev = &stdev(input, 1);
    //af_print!("a1_stdev:", a1_stdev);
    div(input, a1_stdev, true)
}