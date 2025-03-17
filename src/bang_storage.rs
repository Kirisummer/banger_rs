use std::collections::HashMap;
use std::fmt;
use toml::Table;

#[derive(Debug)]
pub struct BangStorage {
    pub bangs: HashMap<String, String>,
    pub default: String,
}

pub type Context = String;

#[derive(Debug)]
pub enum Kind {
    Missing(Context),
    WrongType(Context),
    InvalidValue(Context),
}

#[derive(Debug)]
pub enum ParseErr {
    DefaultBang(Kind),
    Bangs(Kind),
    Bang(Kind),
    Query(Kind),
    Aliases(Kind),
    Alias(Kind),
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error: String = match self {
            ParseErr::DefaultBang(kind) => match kind {
                Kind::Missing(_) => "`default` is missing".to_string(),
                Kind::WrongType(ctx) => format!("`default` is not a string: {}", ctx),
                Kind::InvalidValue(ctx) => format!("`default` is not a defined alias: {}", ctx),
            },
            ParseErr::Bangs(kind) => match kind {
                Kind::Missing(_) => "`bangs` is missing".to_string(),
                Kind::WrongType(ctx) => format!("`bangs` is not an array: {}", ctx),
                Kind::InvalidValue(_) => format!("Impossible message: {:?}", self),
            },
            ParseErr::Bang(kind) => match kind {
                Kind::Missing(_) => format!("Impossible message: {:?}", self),
                Kind::WrongType(ctx) => format!("Bang entry is not a table: {}", ctx),
                Kind::InvalidValue(ctx) => format!("Bang entry has excessive items: {}", ctx),
            },
            ParseErr::Query(kind) => match kind {
                Kind::Missing(ctx) => format!("`query` is missing from bang table: {}", ctx),
                Kind::WrongType(ctx) => format!("`query` is not a string: {}", ctx),
                Kind::InvalidValue(ctx) => format!("`query` has invalid format: {}", ctx),
            },
            ParseErr::Aliases(kind) => match kind {
                Kind::Missing(ctx) => format!("`aliases` is missing from bang table: {}", ctx),
                Kind::WrongType(ctx) => format!("`aliases` is not an array: {}", ctx),
                Kind::InvalidValue(_) => format!("Impossible message: {:?}", self),
            },
            ParseErr::Alias(kind) => match kind {
                Kind::WrongType(ctx) => format!("Alias is not a string: {}", ctx),
                _ => format!("Impossible message: {:?}", self),
            },
        };
        write!(f, "{}", error)
    }
}

impl BangStorage {
    pub fn from_table(table: &Table) -> Result<BangStorage, ParseErr> {
        let mut alias_map = HashMap::new();

        let bang_entries = table
            .get("bangs")
            .ok_or(ParseErr::Bangs(Kind::Missing(String::new())))?
            .as_array()
            .ok_or(ParseErr::Bangs(Kind::WrongType(table["bangs"].to_string())))?;

        for bang_entry in bang_entries {
            let bang_table = bang_entry
                .as_table()
                .ok_or(ParseErr::Bang(Kind::WrongType(bang_entry.to_string())))?;

            let query = bang_table
                .get("query")
                .ok_or(ParseErr::Query(Kind::Missing(bang_table.to_string())))?
                .as_str()
                .ok_or(ParseErr::Query(Kind::WrongType(
                    bang_table["query"].to_string(),
                )))?;

            let aliases = bang_table
                .get("aliases")
                .ok_or(ParseErr::Aliases(Kind::Missing(bang_table.to_string())))?
                .as_array()
                .ok_or(ParseErr::Aliases(Kind::WrongType(
                    bang_table["aliases"].to_string(),
                )))?;

            if bang_table.len() != 2 {
                let extra_items: Vec<String> = bang_table
                    .keys()
                    .filter(|key: &&String| *key != "aliases" || *key != "query")
                    .cloned()
                    .collect();
                return Result::Err(ParseErr::Bang(Kind::InvalidValue(format!(
                    "{:?}",
                    extra_items
                ))));
            }

            for alias_entry in aliases {
                let alias_str = alias_entry
                    .as_str()
                    .ok_or(ParseErr::Alias(Kind::WrongType(alias_entry.to_string())))?;
                alias_map.insert(alias_str.to_string(), query.to_string());
            }
        }

        let default = table
            .get("default")
            .ok_or(ParseErr::DefaultBang(Kind::Missing(String::new())))?
            .as_str()
            .ok_or(ParseErr::DefaultBang(Kind::WrongType(
                table["default"].to_string(),
            )))?
            .to_string();

        if !alias_map.contains_key(&default) {
            return Result::Err(ParseErr::DefaultBang(Kind::InvalidValue(default)));
        }

        Ok(BangStorage {
            bangs: alias_map,
            default: default,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success() {
        const CONTENT: &str = "
            default='duckduckgo'

            [[bangs]]
            aliases = ['duckduckgo', 'ddg']
            query = 'https://duckduckgo.com/?q={}'

            [[bangs]]
            aliases = ['вікі', 'в', 'ukwiki']
            query = 'https://uk.wikipedia.org/w/?search={}'";
        let table: Table = CONTENT.parse().unwrap();
        let storage = BangStorage::from_table(&table).unwrap();
        assert_eq!(storage.default, "duckduckgo");
        assert_eq!(
            storage.bangs,
            HashMap::<String, String>::from([
                (
                    "duckduckgo".to_string(),
                    "https://duckduckgo.com/?q={}".to_string()
                ),
                (
                    "ddg".to_string(),
                    "https://duckduckgo.com/?q={}".to_string()
                ),
                (
                    "вікі".to_string(),
                    "https://uk.wikipedia.org/w/?search={}".to_string()
                ),
                (
                    "в".to_string(),
                    "https://uk.wikipedia.org/w/?search={}".to_string()
                ),
                (
                    "ukwiki".to_string(),
                    "https://uk.wikipedia.org/w/?search={}".to_string()
                ),
            ])
        );
    }

    mod default {
        use super::*;

        #[test]
        fn missing() {
            const CONTENT: &str = "
                [[bangs]]
                aliases = ['duckduckgo', 'ddg']
                query = 'https://duckduckgo.com/?q={}'";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::DefaultBang(Kind::Missing(_))));
        }

        #[test]
        fn wrong_type() {
            const CONTENT: &str = "
                default = 123
                [[bangs]]
                aliases = ['duckduckgo', 'ddg']
                query = 'https://duckduckgo.com/?q={}'";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::DefaultBang(Kind::WrongType(_))));
        }

        #[test]
        fn invalid() {
            const CONTENT: &str = "
                default = 'dddg'
                [[bangs]]
                aliases = ['duckduckgo', 'ddg']
                query = 'https://duckduckgo.com/?q={}'";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(
                error,
                ParseErr::DefaultBang(Kind::InvalidValue(_))
            ));
        }
    }

    mod bangs {
        use super::*;

        #[test]
        fn missing() {
            const CONTENT: &str = "default = 'ddg'";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::Bangs(Kind::Missing(_))));
        }

        #[test]
        fn wrong_type() {
            const CONTENT: &str = "
                default = 'ddg'
                bangs = 123";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::Bangs(Kind::WrongType(_))));
        }
    }

    mod bang {
        use super::*;

        #[test]
        fn wrong_type() {
            const CONTENT: &str = "
                default = 'ddg'
                bangs = [123]";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::Bang(Kind::WrongType(_))));
        }

        #[test]
        fn extra_items() {
            const CONTENT: &str = "
                default = 'ddg'
                [[bangs]]
                query = 'https://duckduckgo.com/?q={}'
                aliases = ['ddg', 'duckduckgo']
                extra = 123";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::Bang(Kind::InvalidValue(_))));
        }
    }

    mod query {
        use super::*;

        #[test]
        fn missing() {
            const CONTENT: &str = "
                default = 'ddg'
                [[bangs]]
                aliases = ['ddg', 'duckduckgo']";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::Query(Kind::Missing(_))));
        }

        #[test]
        fn wrong_type() {
            const CONTENT: &str = "
                default = 'ddg'
                [[bangs]]
                query = 123
                aliases = ['ddg', 'duckduckgo']";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::Query(Kind::WrongType(_))));
        }
    }

    mod aliases {
        use super::*;

        #[test]
        fn missing() {
            const CONTENT: &str = "
                default = 'ddg'
                [[bangs]]
                query = 'https://duckduckgo.com/?q={}'";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::Aliases(Kind::Missing(_))));
        }

        #[test]
        fn wrong_type() {
            const CONTENT: &str = "
                default = 'ddg'
                [[bangs]]
                query = 'https://duckduckgo.com/?q={}'
                aliases = 123";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::Aliases(Kind::WrongType(_))));
        }
    }

    mod alias {
        use super::*;

        #[test]
        fn wrong_type() {
            const CONTENT: &str = "
                default = 'ddg'
                [[bangs]]
                query = 'https://duckduckgo.com/?q={}'
                aliases = ['ddg', 'duckduckgo', 123]";
            let table: Table = CONTENT.parse().unwrap();
            let error = BangStorage::from_table(&table).unwrap_err();
            assert!(matches!(error, ParseErr::Alias(Kind::WrongType(_))));
        }
    }
}
