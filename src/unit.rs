#[derive(PartialEq, Clone, Debug)]
pub struct Unit {
    // Status
    pub name: String,
    pub alive: bool,
    pub metal: f32,
    pub energy: f32,
    // Since we do not implement attacking, these are not even required.
    // health: f32,
    // maxhealth: f32,
    
    // Unit actions
    pub buildpower: f32,
    pub build_target: Option<usize>, // Points to target in world unit list

    // Unit construction
    pub buildtime: f32,
    pub m_build_cost: f32,
    pub e_build_cost: f32,
    
    // Production
    pub e_cost_per_second: f32,
    pub e_per_second: f32,
    pub wind_e_per_second: f32,
    pub e_storage: f32,
    pub m_per_second: f32,
    pub m_storage: f32,
}


impl Unit {
    // Create a new unconstructed unit.
    pub fn new_unconstructed(m_cost: f32, e_cost: f32, buildtime: f32) -> Unit {
        Unit {
            name: "Unnamed".to_string(),
            alive: false,
            metal: 0.0,
            energy: 0.0,
            buildpower: 0.0,
            build_target: None,
            buildtime,
            m_build_cost: m_cost,
            e_build_cost: e_cost,
            e_cost_per_second: 0.0,
            e_per_second: 0.0,
            wind_e_per_second: 0.0,
            e_storage: 0.0,
            m_per_second: 0.0,
            m_storage: 0.0,
        }
    }


    // Construct the unit
    pub fn construct(&mut self) {
        self.metal = self.m_build_cost;
        self.energy = self.e_build_cost;
        self.alive = true;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_unconstructed() {
        let unit = Unit::new_unconstructed(10.0, 50.0, 1000.0);
        assert!(!unit.alive);
        assert_eq!(unit.build_target, None);
        assert_abs_diff_eq!(unit.m_build_cost, 10.0);
        assert_abs_diff_eq!(unit.e_build_cost, 50.0);
        assert_abs_diff_eq!(unit.buildtime, 1000.0);
    }


    #[test]
    fn test_contruction() {
        let mut unit = Unit::new_unconstructed(10.0, 50.0, 1000.0);
        unit.construct();
        
        assert!(unit.alive);
        assert_abs_diff_eq!(unit.metal, 10.0);
        assert_abs_diff_eq!(unit.energy, 50.0);
    }
}