use cw_storage_plus::Item;

pub const BOND_DENOM: Item<String> = Item::new("bond_denom");

pub const ENDING_TIME: Item<u64> = Item::new("ending_time");
