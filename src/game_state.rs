use crate::unit::Unit;
use crate::world_params::WorldParams;

// State of the game
pub struct GameState {
    pub units: Vec<Unit>,
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
            world_params,
            energy,
            metal,
            wind_strength: 25.0,
            time: 0.0,
        }
    }


    // Takes a unit template as argument. 
    pub fn add_completed_unit(&mut self, mut unit: Unit) {
        unit.metal = unit.m_build_cost;
        unit.energy = unit.e_build_cost;
        unit.alive = true;

        self.units.push(unit);
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

        // Clamp the stored resources.
        let max_metal = self.metal_storage();
        let max_energy = self.energy_storage();
        self.metal = self.metal.min(max_metal);
        self.energy = self.energy.min(max_energy);

        self.time += dt;
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
        state.add_completed_unit(com);

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
        state.add_completed_unit(com);

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
        state.add_completed_unit(wind);

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
        state.add_completed_unit(wind.clone());
        state.add_completed_unit(wind);

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
        state.add_completed_unit(mex);

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
        state.add_completed_unit(mex.clone());
        state.add_completed_unit(mex);


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
}