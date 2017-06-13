#[macro_use(mem_info)]
extern crate arrayfire as af;

use std::time::Instant;
use std::num::cast::ToPrimitive;
use af::*;

static NUM_CANDIDATES: u64 = 3;
static SAMPLES: u64 = 20;//_000_000;

#[allow(unused_must_use)]
#[allow(unused_variables)]
fn main() {
    println!("Starting...");
    set_device(0);
    info();
    let dims = Dim4::new(&[SAMPLES, 1, 1, 1]);

    let num_lies = 3;
    let xcand = Dim4::new(&[1, NUM_CANDIDATES, 1, 1]);


    //174245535
    //310368907
    let start = Instant::now();

    mem_info!("Before benchmark");
    {
        let normalized_a1a2 = &get_adversary_ballots_combined();
        af_print!("a1 + a2 normalized:", normalized_a1a2);

        // protagonist's info
//        let honest_middle = &constant::<f32>(0.6, dims);
//        let top = &constant::<f32>(0.0, dims);
//        let bottom = &constant::<f32>(1.0, dims);
//        let utilities = &join_many![1; top, honest_middle, bottom];
        let values: [f32; 3] = [0.0, 0.6, 1.0];
        let utilities = &Array::new(&values, Dim4::new(&[1, 3, 1, 1]));
        af_print!("honest: ", utilities);


        //calculate utility total of honest strat
        let honest_util = {
            let normalized_honest = &normalize_ballot(utilities);
            af_print!("honest normalized: ", normalized_honest);
            let result = &add(normalized_a1a2, normalized_honest, true);
            af_print!("result: ", result);
            let winners = &(imax(result, 1).1);
            af_print!("winners: ", winners);
            let lkup = &histogram(winners, 3, 0.0, 2.0);
            af_print!("lookup: ", lkup);
            let mut counts = [0, 0, 0];
            lkup.host(&mut counts);
            println!("counts: {:?}", counts);


            values[1] * (counts[1].to_usize() as f32) + counts[2]
        };
        println!("here's the utility sum for honest strat: {}", honest_util);

        //        let a1_std_div = &std(x, x, false);
        //        let ysqrd = &mul(y, y, false);
        //        let xplusy = &add(xsqrd, ysqrd, false);
        //        let root = &sqrt(xplusy);
        //        let cnst = &constant(1, dims);
        //        let (real, imag) = sum_all(&le(root, cnst, false));
        //        let pi_val = real*4.0/(SAMPLES as f64);

    }

    mem_info!("After benchmark");
}

fn get_adversary_ballots_combined() -> Array {
    let dimsxcand = Dim4::new(&[SAMPLES, NUM_CANDIDATES, 1, 1]);
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