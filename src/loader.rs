use std::{collections::HashMap, error::Error, fs, path::Path};

use mlua::prelude::*;
use mlua::Value;

use crate::unit::Unit;


fn get_string_or(map: &HashMap<String, Value>, key: &str, default: &str) -> Result<String, Box<dyn Error>> {
    let errmsg = format!("Attempted to parse invalid string for {}.", key);
    Ok(match map.get(key) {
        Some(v) => v.as_string().ok_or(errmsg)?.to_string_lossy(),
        None => default.to_string(),
    })
}

fn get_float_or(map: &HashMap<String, Value>, key: &str, default: f32) -> Result<f32, Box<dyn Error>> {
    let errmsg = format!("Attempted to parse invalid float for {}.", key);
    Ok(match map.get(key) {
        Some(v) => match v {
            LuaValue::Integer(_) => v.as_i32().ok_or(errmsg)? as f32,
            LuaValue::Number(_)  => v.as_f32().ok_or(errmsg)?,
            _ => return Err(errmsg.into())
        }
        None => default,
    })
}

fn get_float(map: &HashMap<String, Value>, key: &str) -> Result<f32, Box<dyn Error>> {
    let errmsg = format!("Attempted to parse invalid float for {}.", key);
    Ok(match map.get(key) {
        Some(v) => match v {
            LuaValue::Integer(_) => v.as_i32().ok_or(errmsg)? as f32,
            LuaValue::Number(_)  => v.as_f32().ok_or(errmsg)?,
            _ => return Err(errmsg.into())
        }
        None => return Err(format!("Required key {} is missing.", key).into()),
    })
}


pub fn load_definition_from_path(definition_path: &Path) -> Result<Unit, Box<dyn Error>> {
    let definition_str = fs::read_to_string(definition_path)?;
    let mut unit = parse_definition(&definition_str)?;
    if unit.name == "Unknown" {
        unit.name = definition_path.file_stem().unwrap().display().to_string()
    }
    Ok(unit)
}


pub fn parse_definition(definition: &str) -> Result<Unit, Box<dyn Error>> {
    let lua = Lua::new();

    let mut defs: HashMap<String, Value> = lua.load(definition).eval()?;
    if defs.len() == 1 {
        defs = HashMap::<String, Value>::from_lua(defs.into_values().next().unwrap(), &lua)?;
    }
    
    let mut e_per_sec = get_float_or(&defs, "energymake", 0.0)?;
    let mut e_cost = get_float_or(&defs, "energyupkeep", 0.0)?;
    if e_cost < 0.0 {
        e_per_sec -= e_cost;
        e_cost = 0.0
    }
    Ok(Unit {
        name: get_string_or(&defs, "name", "Unknown")?,
        alive: false,
        metal: 0.0,
        energy: 0.0,
        buildtime: get_float(&defs, "buildtime")?,
        m_build_cost: get_float(&defs, "metalcost")?,
        e_build_cost: get_float(&defs, "energycost")?,
        buildpower: get_float_or(&defs, "workertime", 0.0)?,
        e_cost_per_second: e_cost,
        e_per_second: e_per_sec,
        wind_e_per_second: get_float_or(&defs, "windgenerator", 0.0)?,
        e_storage: get_float_or(&defs, "energystorage", 0.0)?,
        m_per_second: get_float_or(&defs, "metalmake", 0.0)?,
        m_storage: get_float_or(&defs, "metalstorage", 0.0)?,
    })
}