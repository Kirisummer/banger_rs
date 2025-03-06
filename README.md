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
Configuration files may used to configure bangs Banger uses,
and set a default bang to be used if there are no bangs in the query.

### Location
Configuration is named `banger.toml`.
It is searched for in [$XDG\_CONFIG\_HOME](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html#variables)
and `/etc`, in mentioned order. (TODO!)

### Format
Banger uses [TOML](https://toml.io).
Configuration file is a valid UTF-8 encoded Unicode document.
Values and keys are case-sensitive.

Configuration file consists of `default` key-value pair, and `bangs`,
an array of tables for each bang.
Every mentioned key-value pair is required.

Value of `default` must be a string that corresponds to one of the bangs
from `bangs` array.

Each bang table consists of `aliases` and `query` pairs.
- `aliases` is an array of bang aliases as strings.
    Each bang must be unique to one bang table.
- `query` is a string that contains a URL, that the user will be redirected to
    when the bang is used. Bangs will be stripped,
    and %s will be replaced with search terms.

### Example configuration
```toml
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

## Building and running
Can be built with `cargo`, for example:
```shell
cargo build
```

Program takes 2 CLI arguments: config file and address to bind to. Example:
```shell
cargo run banger.toml 0.0.0.0:8080
```

