#[derive(PartialEq, Clone, Debug)]
pub struct Unit {
    // Status
    // Since we do not implement attacking, these are not even required.
    // health: f32,
    // maxhealth: f32,
    pub name: String,
    pub alive: bool,
    pub metal: f32,
    pub energy: f32,
    
    // Build
    pub buildtime: f32,
    pub m_build_cost: f32,
    pub e_build_cost: f32,
    pub buildpower: f32,
    
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
            buildtime,
            m_build_cost: m_cost,
            e_build_cost: e_cost,
            buildpower: 0.0,
            e_cost_per_second: 0.0,
            e_per_second: 0.0,
            wind_e_per_second: 0.0,
            e_storage: 0.0,
            m_per_second: 0.0,
            m_storage: 0.0,
        }
    }
}