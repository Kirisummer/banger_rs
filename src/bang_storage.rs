use std::collections::HashMap;
use toml::Table;

pub struct BangStorage {
    pub bangs: HashMap<String, String>,
    pub default: String,
}

impl BangStorage {
    pub fn from_table(table: &Table) -> Result<BangStorage, String> {
        let mut alias_map = HashMap::new();

        let bang_entries = table
            .get("bangs")
            .ok_or("`bangs` array is missing")?
            .as_array()
            .ok_or(format!("`bangs` is not an array: {:?}", table["bangs"]))?;

        for bang_entry in bang_entries {
            let bang_table = bang_entry
                .as_table()
                .ok_or(format!("Bang entry is not a table: {:?}", bang_entry))?;

            let query = bang_table
                .get("query")
                .ok_or(format!(
                    "`query` is missing in bang table: {:?}",
                    bang_table
                ))?
                .as_str()
                .ok_or(format!(
                    "`query` is not a string: {:?}",
                    bang_table["query"]
                ))?;

            let aliases = bang_table
                .get("aliases")
                .ok_or(format!(
                    "`aliases` is missing in bang table: {:?}",
                    bang_table
                ))?
                .as_array()
                .ok_or(format!(
                    "`aliases` is not an array: {:?}",
                    bang_table["aliases"]
                ))?;

            for alias_entry in aliases {
                let alias_str = alias_entry
                    .as_str()
                    .ok_or(format!("Alias is not a string: {}", alias_entry))?;
                alias_map.insert(alias_str.to_string(), query.to_string());
            }
        }

        let default = table
            .get("default")
            .ok_or("`default` is missing")?
            .as_str()
            .ok_or(format!("`default` is not a string: {:?}", table["default"]))?
            .to_string();

        if !alias_map.contains_key(&default) {
            return Result::Err(format!("Default bang is not a defined alias: {}", default));
        }

        Ok(BangStorage {
            bangs: alias_map,
            default: default,
        })
    }
}
