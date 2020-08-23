
use bkamins_sir_abm::julia_reimpl::Environment;


fn main() {
    // For basic benchmarking, run the default scenario for ten times
    for _ in 0..10 {
        let mut e = Environment::init(2000, 10, 21, 0.05, 100, 100);
        let states_record = e.run();
    }
}
