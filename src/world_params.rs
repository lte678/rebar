// Contains parameters that affect all units, like global decay rate.
#[derive(Clone)]
pub struct WorldParams {
    pub decay_delay: f32,
    pub decay_rate: f32,
    pub start_metal: f32,
    pub base_metal_storage: f32,
    pub start_energy: f32,
    pub base_energy_storage: f32,
}


impl Default for WorldParams {
    // Defaults taken from BAR
    fn default() -> Self {
        DEFAULT_WORLD_PARAMS.clone()
    }
}


pub const DEFAULT_WORLD_PARAMS: WorldParams = WorldParams {
    decay_delay: 9.0,
    decay_rate: 0.03,
    start_metal: 1000.0,
    base_metal_storage: 500.0,
    start_energy: 1000.0,
    base_energy_storage: 500.0,
};