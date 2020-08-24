//!
//! Things to try out or expand on
//!
//! - Plot the states on the grid itself as to see how the spread is happening
//! - Count how many times each cell has been occupied
//! - Find out by how many agents has any given cell been occupied with at any given time?
//!
//!
//!
//! This is a strict Rust implementation of the presented Julia code in [bkamins' SIR blogpost](https://bkamins.github.io/julialang/2020/08/22/sir.html).
use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentType {
    /// Susceptible
    AgentS,
    /// Infected
    AgentI,
    /// Recovered
    AgentR,
    /// Dead
    AgentD,
}

#[derive(Debug, Clone)]
struct Agent {
    /// Location of an gent in x-dimension
    x: usize,
    /// Location of an gent in y-dimension
    y: usize,
    /// Type of an agent (state of an agent)
    agent_type: AgentType,
    /// Moment in time when agent entered `type`
    tick: usize,
}

impl Agent {
    pub fn die(&mut self, tick: usize) {
        self.agent_type = AgentType::AgentD;
        self.tick = tick;
    }
    pub fn recover(&mut self, tick: usize) {
        self.agent_type = AgentType::AgentR;
        self.tick = tick;
    }
    pub fn infect(&mut self, tick: usize) {
        self.agent_type = AgentType::AgentI;
        self.tick = tick;
    }

    pub fn move_agent(&mut self, grid_dimension: (usize, usize)) {
        if let AgentType::AgentD = self.agent_type {
        } else {
            let mut rng = thread_rng();
            let next_position_sampler = rand_distr::Uniform::new_inclusive(0, 1);
            let negative_sampler = rand::distributions::Bernoulli::new(0.5).unwrap();

            self.x = if rng.sample(negative_sampler) {
                self.x.wrapping_add(rng.sample(next_position_sampler)) % grid_dimension.0
            } else {
                self.x.saturating_sub(rng.sample(next_position_sampler)) % grid_dimension.0
            };
            self.y = if rng.sample(negative_sampler) {
                self.y.wrapping_add(rng.sample(next_position_sampler)) % grid_dimension.1
            } else {
                self.y.saturating_sub(rng.sample(next_position_sampler)) % grid_dimension.1
            };
        }
    }
}

/// World that the agents reside within
pub struct Environment {
    /// For each cell of in the grid, a vector of numbers of agents currently occupying a given cell
    // Note: We first attempt an implementation that relies on *maps
    grid: HashMap<(usize, usize), Vec<usize>>,
    grid_size: (usize, usize),
    agents: Vec<Agent>,
    /// Duration of agents within infected state
    duration: usize,
    /// Probability of death of an agent after duration of infection has elapsed.
    p_death: f64,
    /// Tally of the current states in the grid
    // stats: BTreeMap<AgentType, usize>,
    stats: TallyStates,
    /// Current time tick
    tick: usize,
}

use rand::prelude::*;

impl Environment {
    #[must_use]
    pub fn init(
        n: usize,
        infected: usize,
        duration: usize,
        p_death: f64,
        xdim: usize,
        ydim: usize,
    ) -> Self {
        let mut grid: HashMap<(usize, usize), Vec<usize>> = HashMap::with_capacity(xdim * ydim);

        let mut rng = thread_rng();
        let rand_loc_x = rand_distr::Uniform::new(0, xdim);
        let rand_loc_y = rand_distr::Uniform::new(0, ydim);

        let agents: Vec<Agent> = (0..n)
            .map(|i| Agent {
                x: rng.sample(rand_loc_x),
                y: rng.sample(rand_loc_y),
                agent_type: if i <= infected {
                    AgentType::AgentI
                } else {
                    AgentType::AgentS
                },
                tick: 0,
            })
            .collect();

        for (index, agent) in agents.iter().enumerate() {
            grid.entry((agent.x, agent.y))
                .and_modify(|x| x.push(index))
                .or_insert_with(|| vec![index]);
        }

        let stats = TallyStates {
            susceptible: n - infected,
            infected,
            recovered: 0,
            dead: 0,
        };

        Self {
            grid,
            grid_size: (xdim, ydim),
            agents,
            duration,
            p_death,
            stats,
            tick: 0,
        }
    }

    pub fn update_type(&mut self) {
        let tick = self.tick;
        let mut rng = thread_rng();
        // note: cannot change agents while also using their present state
        // let past_agents = self.agents.clone();
        for i in 0..self.agents.len() {
            if let AgentType::AgentI = self.agents[i].agent_type {
                if tick - self.agents[i].tick > self.duration {
                    if rng.gen_bool(self.p_death) {
                        self.agents[i].die(tick)
                    } else {
                        self.agents[i].recover(tick)
                    }
                } else {
                    if tick == self.agents[i].tick {
                        continue;
                    }

                    for j in self.grid[&(self.agents[i].x, self.agents[i].y)]
                        .clone()
                        .into_iter()
                    {
                        if let AgentType::AgentS = self.agents[j].agent_type {
                            self.agents[j].infect(tick);
                        }
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn get_statistics(&self) -> TallyStates {
        self.agents
            .iter()
            .fold(TallyStates::default(), |mut acc, x| {
                match x.agent_type {
                    AgentType::AgentS => {
                        acc.susceptible += 1;
                    }
                    AgentType::AgentI => {
                        acc.infected += 1;
                    }
                    AgentType::AgentR => {
                        acc.recovered += 1;
                    }
                    AgentType::AgentD => {
                        acc.dead += 1;
                    }
                };
                acc
            })
    }

    pub fn run(&mut self) -> Vec<TallyStates> {
        // max ticks for the default scenario is 300 ticks
        let mut stats_ticks = vec![self.stats.clone()];

        while self.stats.infected > 0 {
            // run while there are infected individuals
            self.tick += 1;
            self.update_type();
            move_all(self);
            //FIXME: maybe this needs to be polled somehow?
            self.stats = self.get_statistics();
            stats_ticks.push(self.stats.clone());
        }

        stats_ticks
    }
}

use soa_derive::StructOfArray;

#[derive(Debug, Default, Clone, StructOfArray)]
#[soa_derive = "Debug"]
pub struct TallyStates {
    susceptible: usize,
    infected: usize,
    recovered: usize,
    dead: usize,
}

fn move_all(
    Environment {
        grid,
        grid_size,
        agents,
        ..
    }: &mut Environment,
) {
    // all agents must move, thus all the locations in the grid are invalid
    // let grid = HashMap::with_capacity(grid.len());
    grid.drain();

    for (i, agent) in agents.iter_mut().enumerate() {
        agent.move_agent(*grid_size);
        grid.entry((agent.x, agent.y))
            .and_modify(|x| x.push(i))
            .or_insert_with(|| vec![i]);
    }
}

/// Return the fraction infected individuals throughout the simulation
fn fraction_infected(l: usize) -> f64 {
    let mut e = Environment::init(2000, 10, l, 0.05, 100, 100);
    e.run();

    1.0 - e.stats.susceptible as f64 / 2000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_environment() {
        // let initial_environment = Environment::init(5, 2, 10, 0.5, 10, 10);
        let initial_environment = Environment::init(25, 2, 10, 0.5, 10, 10);
        println!("Agents:\n\t{:#?}", initial_environment.agents);
        println!("Grid:\n\t{:#?}", initial_environment.grid);
        println!("Stats/State tally:\n\t{:?}", initial_environment.stats);
    }

    #[test]
    fn test_mod1() {
        // assert_eq!(0 % 10, 10);
        // assert_eq!(11 % 10, 1);
        use num::integer::mod_floor as mod1;

        assert_eq!(mod1(0, 10), 10);
        assert_eq!(mod1(11, 10), 1);
    }

    #[test]
    fn test_running_the_model() {
        let mut e = Environment::init(2000, 10, 21, 0.05, 100, 100);
        let states_record = e.run();
        use plotly::{Plot, Scatter};
        use std::iter::FromIterator;

        let ticks: Vec<_> = (0..states_record.len()).collect();
        let soa_records: TallyStatesVec = TallyStatesVec::from_iter(states_record.into_iter());

        let susceptible_trace =
            Scatter::new(ticks.clone(), soa_records.susceptible).name("susceptible");
        let infected_trace = Scatter::new(ticks.clone(), soa_records.infected).name("infected");
        let recovered_trace = Scatter::new(ticks.clone(), soa_records.recovered).name("recovered");
        let dead_trace = Scatter::new(ticks, soa_records.dead).name("dead");

        let mut states_plots = Plot::new();
        states_plots.add_trace(susceptible_trace);
        states_plots.add_trace(infected_trace);
        states_plots.add_trace(recovered_trace);
        states_plots.add_trace(dead_trace);

        states_plots.show();

        // Annotate the legends
    }

    #[test]
    fn test_fraction_infected() {
        let len = 5..=30;
        let runs = 16;

        let inf = len
            .clone()
            .map(|l| (1..=runs).map(|_r| fraction_infected(l)).sum::<f64>() / runs as f64)
            .collect::<Vec<_>>();

        use plotly::{Plot, Scatter};
        let mut fraction_plot = Plot::new();

        let fraction_trace =
            Scatter::new(len.collect::<Vec<_>>(), inf).name("fraction of infected");
        fraction_plot.add_trace(fraction_trace);

        fraction_plot.show();
    }
}
