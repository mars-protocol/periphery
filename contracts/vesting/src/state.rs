use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::msg::{Config, Position};

pub const CONFIG: Item<Config<Addr>> = Item::new("config");

pub const POSITIONS: Map<&Addr, Position> = Map::new("positions");
