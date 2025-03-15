use std::env::{args, var};
use std::path::{Path, PathBuf};

pub struct Env {
    cli_config: Option<PathBuf>,
    home_dir: Option<PathBuf>,
    xdg_config_home: Option<PathBuf>,
    xdg_config_dirs: Vec<PathBuf>,
    sysconfdir: Option<PathBuf>,
    binary_path: PathBuf,
}

impl Env {
    pub fn new(cli_config: Option<PathBuf>) -> Self {
        Env {
            cli_config: cli_config,
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
    match env.cli_config {
        Some(path) if path.is_file() => return Some(path),
        _ => (),
    }

    fn try_join(path: &Path, config_subpath: &Path) -> Option<PathBuf> {
        if path.is_dir() {
            let config_path = path.join(config_subpath);
            if config_path.is_file() {
                return Some(config_path);
            }
        }
        None
    }

    const CONFIG_NAME: &str = "banger.toml";
    let config_subpath: PathBuf = ["banger", CONFIG_NAME].iter().collect();
    if let Some(path) = env.xdg_config_home {
        if let Some(config) = try_join(path.as_path(), &config_subpath) {
            return Some(config);
        }
    }
    if let Some(path) = env.home_dir {
        if let Some(config) = try_join(path.as_path(), &config_subpath) {
            return Some(config);
        }
    }
    for path in env.xdg_config_dirs {
        if let Some(config) = try_join(path.as_path(), &config_subpath) {
            return Some(config);
        }
    }
    if let Some(path) = env.sysconfdir {
        if let Some(config) = try_join(path.as_path(), &config_subpath) {
            return Some(config);
        }
    }
    if let Some(config) = try_join(Path::new("/etc/xdg"), &config_subpath) {
        return Some(config);
    }
    if let Some(config) = try_join(
        env.binary_path.parent().unwrap_or(Path::new("/")),
        Path::new(CONFIG_NAME),
    ) {
        return Some(config);
    }
    None
}
