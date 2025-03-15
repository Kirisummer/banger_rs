use std::env::{args, var};
use std::path::{Path, PathBuf};

pub struct Env {
    /// Path to banger config
    banger_config: Option<PathBuf>,
    /// Absolute path to home directory
    home_dir: Option<PathBuf>,
    /// Absolute path to user config directory
    xdg_config_home: Option<PathBuf>,
    /// List of absolute paths to user config directories
    xdg_config_dirs: Vec<PathBuf>,
    /// Absolute path to system config directory
    sysconfdir: Option<PathBuf>,
    /// Path to binary
    binary_path: PathBuf,
}

impl Env {
    pub fn new() -> Self {
        Env {
            banger_config: var("BANGER_CONFIG").ok().as_deref().map(PathBuf::from),
            home_dir: Self::parse_absolute(var("HOME").ok().as_deref()),
            xdg_config_home: Self::parse_absolute(var("XDG_CONFIG_HOME").ok().as_deref()),
            xdg_config_dirs: Self::parse_absolute_dirlist(var("XDG_CONFIG_DIRS").ok().as_deref()),
            sysconfdir: Self::parse_absolute(var("sysconfdir").ok().as_deref()),
            binary_path: PathBuf::from(args().nth(0).unwrap()),
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
}

pub fn xdg_config_location(env: Env) -> Option<PathBuf> {
    fn try_join(path: &Path, config_subpath: &Path) -> Option<PathBuf> {
        if path.is_dir() {
            let config_path = path.join(config_subpath);
            if config_path.is_file() {
                return Some(config_path);
            }
        }
        None
    }

    macro_rules! try_return_join {
        ($path:expr, $config_subpath:expr) => {
            if let Some(config) = try_join($path, $config_subpath) {
                return Some(config);
            }
        };
    }

    const CONFIG_NAME: &str = "banger.toml";
    let config_subpath: PathBuf = ["banger", CONFIG_NAME].iter().collect();
    if let Some(path) = env.banger_config {
        if path.is_file() {
            return Some(path);
        }
    }
    if let Some(path) = env.xdg_config_home {
        try_return_join!(path.as_path(), &config_subpath);
    }
    if let Some(path) = env.home_dir {
        try_return_join!(path.join(".config").as_path(), &config_subpath);
    }
    for path in env.xdg_config_dirs {
        try_return_join!(path.as_path(), &config_subpath);
    }
    if let Some(path) = env.sysconfdir {
        try_return_join!(path.join("xdg").as_path(), &config_subpath);
    }
    try_return_join!(Path::new("/etc/xdg"), &config_subpath);
    if let Some(path) = env.binary_path.parent() {
        try_return_join!(path, Path::new(CONFIG_NAME));
    }
    None
}
