# A basic SIR model using agent-based approach in Julia / Rust

This repository contains Rust reimplementation of the code for an SIR agent-based model. See [bkamins post](https://bkamins.github.io/julialang/2020/08/22/sir.html) for more details.

This crate contains the code to run and display the plots that are shown in the aforementioned blog
through tests. Thus execute the command `cargo test --release` to see these results.

## Differences between Julia and Rust implementations

- Using mutable references for `die`, `infect`, `move`, and `recover`. 

In the Julia code, there are consuming functions, `die`, `infect`, `move`, and `recover`,
in which they take ownership of the current agent, and create a new agent with the altered
field. This is done through mutable references in the Rust implementation.
This did not alter the computational performance of the code.

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| Pure re-implementation | 2.310 ± 0.171 | 2.106 | 2.620 | 1.00 |
| Mutable references instead immutable operations | 2.164 ± 0.135 | 1.941 | 2.399 | 1.00 |

: Ignore the Relative variable, as these rows come from different runs. This was done using [`hyperfine`](https://github.com/sharkdp/hyperfine).

- `grid` is not a matrix but a `HashMap` also known as a dictionary.

## TODO

- [ ] Displaying the state of the system for each tick
- [ ] Parrallelising using `rayon` maybe?
- [ ] Right now, the modulus being using in Rust impl. is not the same as the `mod1` available in Julia.
There is a test that shows the difference.
- [ ] Benchmark the performance between Julia 1.4 and 1.5 of this simulation.
- [ ] Benchmark Rust vs. Julia implementation.
