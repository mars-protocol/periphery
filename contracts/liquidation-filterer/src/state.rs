use crate::types::Config;
use cw_storage_plus::Item;

// keys (for singleton)
pub const CONFIG: Item<Config> = Item::new("config");
