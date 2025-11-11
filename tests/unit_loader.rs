use std::path::PathBuf;

use rebar::{loader::load_definition_from_path, unit::Unit};

#[test]
fn load_wind() {
    let unit_def_path = PathBuf::from("tests/unitdefs/WindGenerator.lua"); 
    let unit = load_definition_from_path(&unit_def_path).unwrap();
    
    let mut expected = Unit::new_unconstructed(40.0, 175.0, 1600.0);
    // Uses file path name
    expected.name = "WindGenerator".to_string();
    expected.wind_e_per_second = 25.0;
    expected.e_storage = 0.5;
    
    assert_eq!(unit, expected);
}


#[test]
fn load_solar() {
    let unit_def_path = PathBuf::from("tests/unitdefs/Solar.lua"); 
    let unit = load_definition_from_path(&unit_def_path).unwrap();
    let mut expected = Unit::new_unconstructed(155.0, 0.0, 2600.0);
    expected.name = "Basic Solar".to_string();
    expected.e_per_second = 20.0;
    expected.e_storage = 50.0;
    
    assert_eq!(unit, expected);
}


#[test]
fn load_commander() {
    let unit_def_path = PathBuf::from("tests/unitdefs/Commander.lua"); 
    let unit = load_definition_from_path(&unit_def_path).unwrap();
    let expected = Unit {
        name: "Commander".to_string(),
        alive: false,
        metal: 0.0,
        energy: 0.0,
        buildpower: 300.0,
        build_target: None,
        buildtime: 75000.0,
        m_build_cost: 2700.0,
        e_build_cost: 26000.0,
        e_cost_per_second: 0.0,
        e_per_second: 30.0,
        wind_e_per_second: 0.0,
        e_storage: 500.0,
        m_per_second: 2.0,
        m_storage: 500.0,
    };
    
    assert_eq!(unit, expected);
}