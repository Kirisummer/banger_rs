# Banger
A service that imitates DuckDuckGo's bangs:
> Bangs are shortcuts that quickly take you to search results on other sites.
> For example, when you know you want to search on another site like Wikipedia
> or Amazon, our bangs get you there fastest.
> A search for !w filter bubble will take you directly to Wikipedia.
> -- [DuckDuckGo !Bangs](https://duckduckgo.com/bangs)

Banger is intended to be used as search engine proxy,
configured as a search engine in browsers.

[License](/LICENSE)

## Configuration
Configuration files are used to configure bangs Banger uses,
set a default bang to be used if there are no bangs in the query,
and define a default address the server will be listening on.

### Location
Configuration is named `banger.toml`. It may be passed with `--config` or `-c` CLI option.
Otherwise, it is searched for in following order:
1. `$BANGER_CONFIG`
2. `$XDG_CONFIG_HOME/banger/banger.toml`
3. `$HOME/.config/banger/banger.toml`
4. `<dir>/banger/banger.toml`, `dir` is taken from `$XDG_CONFIG_DIRS` list, where directories are split by `:`
5. `$sysconfdir/xdg/banger/banger.toml`
6. `/etc/xdg/banger/banger.toml`
7. `<directory_with_binary>/banger.toml`

Names with `$` are environment variables.
For locations 2-6, Banger attempts to implement [XDG base directory specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html#variables).

### Format
Banger uses [TOML](https://toml.io).
Configuration file is a valid UTF-8 encoded Unicode document.
Values and keys are case-sensitive.

Configuration file consists of `default` and `address` key-value pairs, and `bangs`,
an array of tables for each bang.
`address` is optional, every other mentioned key-value pair is required.

Value of `address` must be a string in format `<IP address>:<port>`.

Value of `default` must be a string that corresponds to one of the bangs
from `bangs` array.

Each bang table consists of `aliases` and `query` pairs.
- `aliases` is an array of bang aliases as strings.
    Each bang must be unique to one bang table.
- `query` is a string that contains a URL, that the user will be redirected to
    when the bang is used. Bangs will be stripped,
    and `{}` will be replaced with search terms.

### Example configuration
```toml
address='127.0.0.1:8080'
default='duckduckgo'

[[bangs]]
aliases = ['duckduckgo', 'ddg']
query = 'https://duckduckgo.com/?q={}'

[[bangs]]
aliases = ['wikipedia', 'wiki', 'w']
query = 'https://en.wikipedia.org/w/?search={}'

[[bangs]]
aliases = ['вікі', 'в', 'ukwiki']
query = 'https://uk.wikipedia.org/w/?search={}'
```

Here, we can see three configurations, with DuckDuckGo selected as a default:
- [DuckDuckGo](https://duckduckgo.com), with two bang aliases and a query
- [Wikipedia](https://en.wikipedia.org), with three bang aliases and a query
- [Ukrainian Wikipedia](https://uk.wikipedia.org), with three bang aliases,
    some with Unicode symbols, and a query

Address is set to 127.0.0.1 with port 8080.

## Installing

### ArchLinux:
```shell
makepkg -si
```

### Manually on \*nix
To install to ~/.cargo/bin:
```shell
cargo install
```
Other paths and options may be chosen: check [`cargo install` documentation](https://doc.rust-lang.org/cargo/commands/cargo-install.html)

Alternatively:
```shell
cargo build --release
cp target/release/banger_rs <somewhere_in_your_path>
```

## Running

### Manually
Program takes 2 optional CLI arguments: config file and address to bind to. Examples:
```shell
# Look up config and take address from config
banger_rs
# Look up config and override address
banger_rs --address 0.0.0.0:8080
# Select config manually
banger_rs -c banger.toml
# Select config manually and override address
banger_rs --config banger.toml --address 0.0.0.0:8080
# Short options may be used
banger_rs -c banger.toml -a 0.0.0.0:8080
```

### As systemd service

#### With package installed
```shell
# Run at system startup
sudo systemctl enable --now banger
# Run at first user login
systemctl --user enable --now banger
```

#### Manually
You may want to edit path to the binary in the service file before running commands.

```shell
# Run at system startup
sudo systemctl enable --now ./banger.service
# Run at first user login
systemctl --user enable --now ./banger.service
```
