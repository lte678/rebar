use std::collections::HashMap;
use std::error::Error;

use approx::abs_diff_eq;

use crate::unit::Unit;
use crate::world_params::WorldParams;

// State of the game
pub struct GameState {
    pub units: Vec<Unit>,
    pub unit_catalog: HashMap<String, Unit>,
    pub world_params: WorldParams,
    pub energy: f32,
    pub metal: f32,
    pub wind_strength: f32,
    pub time: f32,
}


impl GameState {
    pub fn new(world_params: WorldParams) -> GameState {
        let energy = world_params.start_energy;
        let metal = world_params.start_metal;
        GameState { 
            units: Vec::new(),
            unit_catalog: HashMap::new(),
            world_params,
            energy,
            metal,
            wind_strength: 25.0,
            time: 0.0,
        }
    }


    // Copies the unit template and constructs it.
    // The unit must first be registered using `register_unit`. 
    pub fn add_completed_unit(&mut self, unit_name: &str) -> Result<usize, Box<dyn Error>> {
        let unit_idx = self.add_unit(unit_name)?;
        self.units[unit_idx].construct();
        Ok(unit_idx)
    }

    
    // Adds a unit by copying its template.
    // The unit must first be registered using `register_unit`. 
    pub fn add_unit(&mut self, unit_name: &str) -> Result<usize, Box<dyn Error>> {
        let unit_template = self.unit_catalog.get(unit_name).ok_or(format!("'{}' is not a known unit.", unit_name))?;
        self.units.push(unit_template.clone());
        Ok(self.units.len() - 1)
    }


    // Make the unit available under this name
    pub fn register_unit(&mut self, name: &str, unit: Unit) {
        self.unit_catalog.insert(name.to_string(), unit);
    }


    pub fn simulate(&mut self, dt: f32) {
        // Energy and metal production
        for unit in &self.units {
            if unit.alive {
                self.energy += dt * unit.e_per_second;
                self.energy += dt * unit.wind_e_per_second.min(self.wind_strength);
            }
        }
        // Let everything consume energy before we clamp the upper storage limits.
        
        // For resource consumption, we technically need to implement the BAR priority system.
        // First, high prio constructors get resources
        // Second, mexes, radar, etc.
        // I believe unit production gets the resources last.
        
        // We will try to imitate the system where the energy is allocated in a binary fashion.
        // It also seems that the order things are built matters, so the fact that an arbitrary unit will be preferred
        // over other ones due to it's iteration order is intended behavior.
        for unit in &self.units {
            if unit.alive {
                let e_consumed = dt * unit.e_cost_per_second;
                if self.energy > e_consumed {
                    self.energy -= e_consumed;
                    // Do things that powered units do, like produce metal.
                    self.metal  += dt * unit.m_per_second;
                }
            }
        }

        // Assign build power
        // We need to use index-based loops since we are modifying the contents of elements different to the one we are looped over.
        for i in 0..self.units.len() {
            if let Some(target_idx) = self.units[i].build_target && self.units[i].alive {
                // This unit is building something
                // The percentage of the target to build in this timestep
                let mut build_step = dt * self.units[i].buildpower / self.units[target_idx].buildtime;
                let remaining = 1.0 - self.units[target_idx].metal / self.units[target_idx].m_build_cost;
                build_step = build_step.min(remaining);
                
                let build_m_cost = build_step * self.units[target_idx].m_build_cost;
                let build_e_cost = build_step * self.units[target_idx].e_build_cost;
                if build_m_cost < self.metal && build_e_cost < self.energy {
                    self.metal -= build_m_cost;
                    self.energy -= build_e_cost;
                    self.units[target_idx].metal += build_m_cost;
                    self.units[target_idx].energy += build_e_cost;
                }

                if abs_diff_eq!(build_step, remaining) {
                    self.units[target_idx].construct();
                    self.units[i].build_target = None;
                }
            }
        }

        // Clamp the stored resources.
        let max_metal = self.metal_storage();
        let max_energy = self.energy_storage();
        self.metal = self.metal.min(max_metal);
        self.energy = self.energy.min(max_energy);

        self.time += dt;
    }


    // Use unit to build a new unit
    pub fn build_unit(&mut self, builder: usize, buildee: &str) -> Result<usize, Box<dyn Error>> {
        // Make sure that the builder is allowed to build the unit
        if !self.units[builder].build_options.contains(buildee) {
            return Err(format!("Constructor cannot build unit '{}'.", buildee).into())
        }

        let buildee_idx = self.add_unit(buildee)?;
        self.units[builder].build_target = Some(buildee_idx);
        Ok(buildee_idx)
    }


    pub fn metal_storage(&self) -> f32 {
        let mut storage: f32 = self.world_params.base_metal_storage;
        for unit in &self.units {
            if unit.alive {
                storage += unit.m_storage;
            }
        }
        storage
    }


    pub fn energy_storage(&self) -> f32 {
        let mut storage: f32 = self.world_params.base_energy_storage;
        for unit in &self.units {
            if unit.alive {
                storage += unit.e_storage;
            }
        }
        storage
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_time() {
        let mut state = GameState::new(WorldParams::default());
        
        assert_abs_diff_eq!(state.time, 0.0);
        state.simulate(1.5);
        assert_abs_diff_eq!(state.time, 1.5);
        state.simulate(0.1);
        assert_abs_diff_eq!(state.time, 1.6);
    }


    #[test]
    fn test_storage() {
        let mut state = GameState::new(WorldParams::default());
        assert_abs_diff_eq!(state.energy_storage(), 500.0);
        assert_abs_diff_eq!(state.metal_storage(), 500.0);

        // Create a commander.
        let mut com = Unit::new_unconstructed(1.0, 1.0, 1.0);
        com.m_storage = 500.0;
        com.e_storage = 500.0;
        state.register_unit("commander", com);
        state.add_completed_unit("commander").unwrap();

        // Test that the state matches the start of a normal game of BAR
        state.simulate(0.01);
        assert_abs_diff_eq!(state.energy_storage(), 1000.0);
        assert_abs_diff_eq!(state.metal_storage(), 1000.0);
        assert_abs_diff_eq!(state.energy, 1000.0);
        assert_abs_diff_eq!(state.metal, 1000.0);

        // Test that removing the commander reduces to 500 storage. (This was tested)
        state.units.clear();
        state.simulate(0.01);
        assert_abs_diff_eq!(state.energy_storage(), 500.0);
        assert_abs_diff_eq!(state.metal_storage(), 500.0);
        assert_abs_diff_eq!(state.energy, 500.0);
        assert_abs_diff_eq!(state.metal, 500.0);
    }


    #[test]
    fn test_resource_generation() {
        let mut state = GameState::new(WorldParams::default());
        state.simulate(2.0);
        assert_abs_diff_eq!(state.energy, 500.0);
        assert_abs_diff_eq!(state.metal, 500.0);

        // Create a commander.
        let mut com = Unit::new_unconstructed(1.0, 1.0, 1.0);
        com.m_storage = 500.0;
        com.e_storage = 500.0;
        com.e_per_second = 30.0;
        com.m_per_second = 2.0;
        state.register_unit("commander", com);
        state.add_completed_unit("commander").unwrap();

        state.simulate(2.0);
        assert_abs_diff_eq!(state.energy, 560.0);
        assert_abs_diff_eq!(state.metal, 504.0);

        state.simulate(250.0);
        assert_abs_diff_eq!(state.energy, 1000.0);
        assert_abs_diff_eq!(state.metal, 1000.0);
    }

    #[test]
    fn test_wind() {
        let mut state = GameState::new(WorldParams::default());
        state.wind_strength = 8.0;
        state.simulate(2.0);
        assert_abs_diff_eq!(state.energy, 500.0);
        assert_abs_diff_eq!(state.metal, 500.0);

        // Add wind
        let mut wind = Unit::new_unconstructed(1.0, 1.0, 1.0);
        wind.wind_e_per_second = 25.0;
        wind.e_storage = 100.0;
        state.register_unit("wind", wind);
        state.add_completed_unit("wind").unwrap();

        // Wind energy should be limited by the minimum of the unit's wind e and environment wind.
        state.simulate(2.0);
        assert_abs_diff_eq!(state.energy, 516.0);
    }


    #[test]
    fn test_double_wind() {
        let mut state = GameState::new(WorldParams::default());
        state.wind_strength = 8.0;
        state.simulate(2.0);
        assert_abs_diff_eq!(state.energy, 500.0);
        assert_abs_diff_eq!(state.metal, 500.0);

        // Add wind
        let mut wind = Unit::new_unconstructed(1.0, 1.0, 1.0);
        wind.wind_e_per_second = 25.0;
        wind.e_storage = 100.0;
        state.register_unit("wind", wind);
        state.add_completed_unit("wind").unwrap();
        state.add_completed_unit("wind").unwrap();

        // Wind energy should be limited by the minimum of the unit's wind e and environment wind.
        state.simulate(2.0);
        assert_abs_diff_eq!(state.energy, 532.0);
    }


    #[test]
    fn test_mex() {
        let mut state = GameState::new(WorldParams::default());
        state.energy = 25.0;
        state.simulate(2.0);
        assert_abs_diff_eq!(state.energy, 25.0);
        assert_abs_diff_eq!(state.metal, 500.0);

        // Create a metal extractor.
        let mut mex: Unit = Unit::new_unconstructed(1.0, 1.0, 1.0);
        mex.m_storage = 50.0;
        mex.e_cost_per_second = 3.0;
        mex.m_per_second = 3.0;
        state.register_unit("mex", mex);
        state.add_completed_unit("mex").unwrap();

        state.simulate(2.0);
        assert_abs_diff_eq!(state.energy, 19.0);
        assert_abs_diff_eq!(state.metal, 506.0);
        state.simulate(6.0);
        assert_abs_diff_eq!(state.energy, 1.0);
        assert_abs_diff_eq!(state.metal, 524.0);
        state.simulate(1.0); // Energy stall
        assert_abs_diff_eq!(state.energy, 1.0);
        assert_abs_diff_eq!(state.metal, 524.0);
        
        state.energy = 100.0;
        state.simulate(1.0);
        assert_abs_diff_eq!(state.metal, 527.0);
    }


    #[test]
    fn test_double_mex() {
        let mut state = GameState::new(WorldParams::default());
        state.energy = 28.0;
        state.simulate(2.0);
        assert_abs_diff_eq!(state.energy, 28.0);
        assert_abs_diff_eq!(state.metal, 500.0);

        // Create a metal extractor.
        let mut mex: Unit = Unit::new_unconstructed(1.0, 1.0, 1.0);
        mex.m_storage = 50.0;
        mex.e_cost_per_second = 3.0;
        mex.m_per_second = 3.0;
        state.register_unit("mex", mex);
        state.add_completed_unit("mex").unwrap();
        state.add_completed_unit("mex").unwrap();

        state.simulate(4.0);
        assert_abs_diff_eq!(state.energy, 4.0);
        assert_abs_diff_eq!(state.metal, 524.0);
        state.simulate(1.0); // Energy stall
        assert_abs_diff_eq!(state.energy, 1.0);
        assert_abs_diff_eq!(state.metal, 527.0);
        state.simulate(1.0); // Energy stall
        assert_abs_diff_eq!(state.energy, 1.0);
        assert_abs_diff_eq!(state.metal, 527.0);
    }


    #[test]
    fn test_build_power() {
        // Create the world
        let mut state = GameState::new(WorldParams::default());
        state.wind_strength = 20.0;
        state.energy = 500.0;
        state.metal = 500.0;

        // Create a commander.
        let mut com = Unit::new_unconstructed(1.0, 1.0, 1.0);
        com.m_storage = 500.0;
        com.e_storage = 500.0;
        com.buildpower = 300.0;
        state.register_unit("commander", com);
        
        // Add an incomplete unit.
        let mut wind = Unit::new_unconstructed(40.0, 175.0, 1600.0);
        wind.wind_e_per_second = 25.0;
        wind.e_storage = 100.0;
        state.register_unit("wind", wind);
        
        state.add_completed_unit("commander").unwrap();
        state.add_unit("wind").unwrap();

        // Assert that the wind is not producing energy, since it is not constructed
        state.simulate(1.0);
        assert_abs_diff_eq!(state.energy, 500.0);
        assert_abs_diff_eq!(state.metal, 500.0);

        // Build the wind. Should take 5.333 seconds
        state.units[0].build_target = Some(1);

        state.simulate(0.5 * (1600.0 / 300.0));
        assert!(!state.units[1].alive);
        assert_abs_diff_eq!(state.units[1].energy, 175.0 * 0.5);
        assert_abs_diff_eq!(state.units[1].metal, 40.0 * 0.5);
        assert_abs_diff_eq!(state.energy, 500.0 - 175.0 * 0.5);
        assert_abs_diff_eq!(state.metal, 500.0 - 40.0 * 0.5);

        state.simulate(0.5 * (1600.0 / 300.0) + 1e-9);
        assert!(state.units[1].alive);
        assert_abs_diff_eq!(state.units[1].energy, 175.0);
        assert_abs_diff_eq!(state.units[1].metal, 40.0);
        assert_abs_diff_eq!(state.energy, 500.0 - 175.0);
        assert_abs_diff_eq!(state.metal, 500.0 - 40.0);

        state.simulate(1.0);
        assert_abs_diff_eq!(state.units[1].energy, 175.0);
        assert_abs_diff_eq!(state.units[1].metal, 40.0);
        assert_abs_diff_eq!(state.energy, 500.0 - 175.0 + 20.0);
        assert_abs_diff_eq!(state.metal, 500.0 - 40.0);

        assert_eq!(state.units[0].build_target, None);
    }


    #[test]
    fn test_build_unit() {
        // Create the world
        let mut state = GameState::new(WorldParams::default());
        state.wind_strength = 20.0;
        state.energy = 500.0;
        state.metal = 500.0;

        let mut com = Unit::new_unconstructed(1.0, 1.0, 1.0);
        com.m_storage = 500.0;
        com.e_storage = 500.0;
        com.buildpower = 300.0;
        state.register_unit("commander", com);
        
        let mut wind = Unit::new_unconstructed(40.0, 175.0, 1600.0);
        wind.wind_e_per_second = 25.0;
        wind.e_storage = 100.0;
        state.register_unit("wind", wind);
        
        
        // Create a commander.
        let com_idx = state.add_completed_unit("commander").unwrap();
        // Produce a unit that the commander may not build
        let err = state.build_unit(com_idx, "wind");
        assert!(matches!(err, Err(_)));
        assert_eq!(state.units.len(), 1);

        // Add the unit to the commander's capabilities
        state.units[0].build_options.insert("wind".to_string());
        state.build_unit(com_idx, "wind").unwrap();
        assert_eq!(state.units.len(), 2);

        // Build the wind. Should take 5.333 seconds
        // It should be automatically selected by the commander.
        state.simulate(0.5 * (1600.0 / 300.0));
        assert!(!state.units[1].alive);
        assert_abs_diff_eq!(state.units[1].energy, 175.0 * 0.5);
        assert_abs_diff_eq!(state.units[1].metal, 40.0 * 0.5);
        assert_abs_diff_eq!(state.energy, 500.0 - 175.0 * 0.5);
        assert_abs_diff_eq!(state.metal, 500.0 - 40.0 * 0.5);
    }
}