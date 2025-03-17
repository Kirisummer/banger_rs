use std::env::{args, var};
use std::path::{Path, PathBuf};

fn try_join(
    path: &Path,
    config_subpath: &Path,
    dir_exists: &dyn Fn(&Path) -> bool,
    file_exists: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    if dir_exists(path) {
        let config_path = path.join(config_subpath);
        if file_exists(&config_path) {
            return Some(config_path);
        }
    }
    None
}

macro_rules! try_return_join {
    ($path:expr, $config_subpath:expr, $dir_exists:expr, $file_exists:expr) => {
        if let Some(config) = try_join($path, $config_subpath, $dir_exists, $file_exists) {
            return Some(config);
        }
    };
}

fn real_env_var(name: &str) -> Option<String> {
    var(name).ok()
}

const CONFIG_SUBDIR: &str = "banger";
const CONFIG_NAME: &str = "banger.toml";

pub struct XdgLookup {
    /// Absolute path to home directory
    home_dir: Option<PathBuf>,
    /// Absolute path to user config directory
    xdg_config_home: Option<PathBuf>,
    /// List of absolute paths to user config directories
    xdg_config_dirs: Vec<PathBuf>,
    /// Absolute path to system config directory
    sysconfdir: Option<PathBuf>,
}

impl XdgLookup {
    fn new(env_var: &dyn Fn(&str) -> Option<String>) -> Self {
        Self {
            home_dir: Self::parse_absolute(env_var("HOME").as_deref()),
            xdg_config_home: Self::parse_absolute(env_var("XDG_CONFIG_HOME").as_deref()),
            xdg_config_dirs: Self::parse_absolute_dirlist(env_var("XDG_CONFIG_DIRS").as_deref()),
            sysconfdir: Self::parse_absolute(env_var("sysconfdir").as_deref()),
        }
    }

    fn parse_absolute(string: Option<&str>) -> Option<PathBuf> {
        match string.map(|value| PathBuf::from(value)) {
            Some(path) if path.is_absolute() => Some(path),
            _ => None,
        }
    }

    fn parse_absolute_dirlist(string: Option<&str>) -> Vec<PathBuf> {
        match string {
            Some(s) => s
                .split(':')
                .map(Option::from)
                .filter_map(Self::parse_absolute)
                .collect::<Vec<PathBuf>>(),
            None => Vec::new(),
        }
    }

    fn lookup(
        &self,
        dir_exists: &dyn Fn(&Path) -> bool,
        file_exists: &dyn Fn(&Path) -> bool,
    ) -> Option<PathBuf> {
        let config_subpath: PathBuf = [CONFIG_SUBDIR, CONFIG_NAME].iter().collect();

        if let Some(path) = &self.xdg_config_home {
            try_return_join!(path.as_path(), &config_subpath, dir_exists, file_exists);
        }
        if let Some(path) = &self.home_dir {
            try_return_join!(
                path.join(".config").as_path(),
                &config_subpath,
                dir_exists,
                file_exists
            );
        }
        for path in &self.xdg_config_dirs {
            try_return_join!(path.as_path(), &config_subpath, dir_exists, file_exists);
        }
        if let Some(path) = &self.sysconfdir {
            try_return_join!(
                path.join("xdg").as_path(),
                &config_subpath,
                dir_exists,
                file_exists
            );
        }
        try_return_join!(
            Path::new("/etc/xdg"),
            &config_subpath,
            dir_exists,
            file_exists
        );

        None
    }
}

pub struct ConfigLookup {
    /// Config location supplied via CLI
    cli_config: Option<PathBuf>,
    /// Config location supplied via environment variable
    env_config: Option<PathBuf>,
    /// XDG-related environment variables
    xdg: XdgLookup,
    /// Path to binary
    binary_path: PathBuf,
}

impl ConfigLookup {
    pub fn new(cli_config: Option<PathBuf>) -> Self {
        Self::new_custom(
            cli_config,
            &real_env_var,
            PathBuf::from(args().nth(0).unwrap()),
        )
    }

    fn new_custom(
        cli_config: Option<PathBuf>,
        env_var: &dyn Fn(&str) -> Option<String>,
        binary_path: PathBuf,
    ) -> Self {
        Self {
            cli_config: cli_config,
            env_config: env_var("BANGER_CONFIG").as_deref().map(PathBuf::from),
            xdg: XdgLookup::new(&env_var),
            binary_path: binary_path,
        }
    }

    pub fn lookup(&self) -> Option<PathBuf> {
        self.lookup_custom(&|path: &Path| path.is_dir(), &|path: &Path| path.is_file())
    }

    fn lookup_custom(
        &self,
        dir_exists: &dyn Fn(&Path) -> bool,
        file_exists: &dyn Fn(&Path) -> bool,
    ) -> Option<PathBuf> {
        if let Some(path) = &self.cli_config {
            if file_exists(path) {
                return Some(path.to_path_buf());
            } else {
                return None;
            }
        }
        if let Some(path) = &self.env_config {
            if file_exists(path) {
                return Some(path.to_path_buf());
            } else {
                return None;
            }
        }
        if let Some(path) = &self.xdg.lookup(dir_exists, file_exists) {
            return Some(path.clone());
        }
        if let Some(path) = &self.binary_path.parent() {
            try_return_join!(path, Path::new(CONFIG_NAME), dir_exists, file_exists);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLI_CONFIG: &str = "cli_banger.toml";
    const ENV_CONFIG: &str = "env_banger.toml";
    const XDG_CONFIG_HOME: &str = "/CONFIG_HOME";
    const HOME: &str = "/HOME";
    const XDG_CONFIG_DIRS: &str = "/CONFIG_DIR1:/CONFIG_DIR2";
    const SYSCONFDIR: &str = "/SYSCONF";

    const XDG_CONFIG_HOME_FILE: &str = "/CONFIG_HOME/banger/banger.toml";
    const HOME_FILE: &str = "/HOME/.config/banger/banger.toml";
    const XDG_CONFIG_DIRS_FILE: &str = "/CONFIG_DIR2/banger/banger.toml";
    const SYSCONFDIR_FILE: &str = "/SYSCONF/xdg/banger/banger.toml";
    const ETC_XDG_FILE: &str = "/etc/xdg/banger/banger.toml";
    const BINARY_DIR_FILE: &str = "/BINARY_DIR/banger.toml";

    const CONFIG_FILES: [&str; 8] = [
        CLI_CONFIG,
        ENV_CONFIG,
        XDG_CONFIG_HOME_FILE,
        HOME_FILE,
        XDG_CONFIG_DIRS_FILE,
        SYSCONFDIR_FILE,
        ETC_XDG_FILE,
        BINARY_DIR_FILE,
    ];

    fn full_env_var(var: &str) -> Option<String> {
        match var {
            "HOME" => Some(HOME.to_string()),
            "XDG_CONFIG_HOME" => Some(XDG_CONFIG_HOME.to_string()),
            "XDG_CONFIG_DIRS" => Some(XDG_CONFIG_DIRS.to_string()),
            "sysconfdir" => Some(SYSCONFDIR.to_string()),
            _ => None,
        }
    }

    #[test]
    fn cli() {
        let lookup = ConfigLookup::new_custom(
            Some(PathBuf::from(CLI_CONFIG)),
            &full_env_var,
            PathBuf::from(BINARY_DIR_FILE),
        );
        let file_exists = |file: &Path| CONFIG_FILES.contains(&file.to_str().unwrap());
        assert_eq!(
            Some(PathBuf::from(CLI_CONFIG)),
            lookup.lookup_custom(&|_| true, &file_exists)
        );
    }

    #[test]
    fn no_cli() {
        let lookup = ConfigLookup::new_custom(
            Some(PathBuf::from(CLI_CONFIG)),
            &full_env_var,
            PathBuf::from(BINARY_DIR_FILE),
        );
        let file_exists = |file: &Path| CONFIG_FILES[1..].contains(&file.to_str().unwrap());
        assert_eq!(None, lookup.lookup_custom(&|_| true, &file_exists));
    }

    #[test]
    fn env() {
        let env_var = |var: &str| match var {
            "BANGER_CONFIG" => Some(ENV_CONFIG.to_string()),
            _ => full_env_var(var),
        };
        let lookup = ConfigLookup::new_custom(None, &env_var, PathBuf::from(BINARY_DIR_FILE));
        let file_exists = |file: &Path| CONFIG_FILES[1..].contains(&file.to_str().unwrap());
        assert_eq!(
            Some(PathBuf::from(ENV_CONFIG)),
            lookup.lookup_custom(&|_| true, &file_exists)
        );
    }

    #[test]
    fn no_env() {
        let env_var = |var: &str| match var {
            "BANGER_CONFIG" => Some(ENV_CONFIG.to_string()),
            _ => full_env_var(var),
        };
        let lookup = ConfigLookup::new_custom(None, &env_var, PathBuf::from(BINARY_DIR_FILE));
        let file_exists = |file: &Path| CONFIG_FILES[2..].contains(&file.to_str().unwrap());
        assert_eq!(None, lookup.lookup_custom(&|_| true, &file_exists));
    }

    #[test]
    fn xdg_config_home() {
        let lookup = ConfigLookup::new_custom(None, &full_env_var, PathBuf::from(BINARY_DIR_FILE));
        let file_exists = |file: &Path| CONFIG_FILES[2..].contains(&file.to_str().unwrap());
        assert_eq!(
            Some(PathBuf::from(XDG_CONFIG_HOME_FILE)),
            lookup.lookup_custom(&|_| true, &file_exists)
        );
    }

    #[test]
    fn home() {
        let lookup = ConfigLookup::new_custom(None, &full_env_var, PathBuf::from(BINARY_DIR_FILE));
        let file_exists = |file: &Path| CONFIG_FILES[3..].contains(&file.to_str().unwrap());
        assert_eq!(
            Some(PathBuf::from(HOME_FILE)),
            lookup.lookup_custom(&|_| true, &file_exists)
        );
    }

    #[test]
    fn xdg_config_dirs() {
        let lookup = ConfigLookup::new_custom(None, &full_env_var, PathBuf::from(BINARY_DIR_FILE));
        let file_exists = |file: &Path| CONFIG_FILES[4..].contains(&file.to_str().unwrap());
        assert_eq!(
            Some(PathBuf::from(XDG_CONFIG_DIRS_FILE)),
            lookup.lookup_custom(&|_| true, &file_exists)
        );
    }

    #[test]
    fn sysconfdir() {
        let lookup = ConfigLookup::new_custom(None, &full_env_var, PathBuf::from(BINARY_DIR_FILE));
        let file_exists = |file: &Path| CONFIG_FILES[5..].contains(&file.to_str().unwrap());
        assert_eq!(
            Some(PathBuf::from(SYSCONFDIR_FILE)),
            lookup.lookup_custom(&|_| true, &file_exists)
        );
    }

    #[test]
    fn etc_xdg() {
        let lookup = ConfigLookup::new_custom(None, &full_env_var, PathBuf::from(BINARY_DIR_FILE));
        let file_exists = |file: &Path| CONFIG_FILES[6..].contains(&file.to_str().unwrap());
        assert_eq!(
            Some(PathBuf::from(ETC_XDG_FILE)),
            lookup.lookup_custom(&|_| true, &file_exists)
        );
    }

    #[test]
    fn binary_path() {
        let lookup = ConfigLookup::new_custom(None, &full_env_var, PathBuf::from(BINARY_DIR_FILE));
        let file_exists = |file: &Path| file.to_str().unwrap() == BINARY_DIR_FILE;
        assert_eq!(
            Some(PathBuf::from(BINARY_DIR_FILE)),
            lookup.lookup_custom(&|_| true, &file_exists)
        );
    }
}
