# anyrun-hunspell

A simple [anyrun](https://github.com/anyrun-org/anyrun) plugin adding spell checking via [Hunspell](http://hunspell.github.io/).

## Usage

Start a query with `:spell` (configurable) to trigger spell checking. 

The expected syntax is `:spell[:DICTIONARY] <WORD> [<WORD2>, ...]`

The optional dictionary must be of the same form as what `hunspell` accepts with it's `-d` option, eg. `en_US`.
Alternatively it can be an alias to a dictionary (see below).

Selecting an entry for a correct word / suggestion will copy the word / suggestion to the clipboard.
Selecting an invalid dictionary, incorrect word or waiting entry will close the runner without copying anything.

## Future plans

- [ ] Multiple dictionaries: Allow to specify multiple dictionaries to use simultaniously in both the runner and in aliases.

Suggestions can be submitted by creating a new issue (prefer gitlab over github if possible).

## Installing

The plugin requires hunspell to be installed as well as any dictionaries for the languages you want to use.
The packages can likely be found by searching for `hunspell` in your package manager.
Otherwise check your distro's documentation for installation instructions.

To check what dictionaries are installed and where `hunspell` will search for them run `hunspell -D`.
This is what the plugin runs internally to verify if the given dictionary exists.

**NOTE**: Dictionaries not detect by `hunspell -D` are unsupported.
The recommended place to add custom dictionaries where hunspell will find them is `/home/your_user/Library/Spelling`.

Clone the repo and run `just install`:

```sh
# clone and enter the repo
git clone https://gitlab.com/udragg/anyrun-hunspell.git && cd anyrun-hunspell

# build the plugin and copy it to ~/.config/anyrun/plugins/
# if just is not installed run the commands in the install recipe manually (or install just)
just install
```

Lastly add `"libhunspell.so"` to your plugins list in the main anyrun `config.toml`.

Optionally run `just install_default_config` to write the default `hunspell.ron` config to `~/.config/anyrun/` or `just install_default_config <path_to_anyrun_config_dir>` if you use a different config directory .

## Configuring

Example of the default configuration.

```ron
# file: <ANYRUN_CONFIG_DIR>/hunspell.ron
Config(
    prefix: ":spell",
    default_language: "en_US",
    max_entries: 15,
    aliases: [
        Alias(
            name: "en",
            dictionary: "en_US",
        ),
    ],
)
```

| Key                | Default                                    | Meaning                                                    |
|--------------------|--------------------------------------------|------------------------------------------------------------|
| `prefix`           | `":spell"`                                 | Prefix to trigger spell checker                            |
| `default_language` | `"en_US"`                                  | Default language if none is specified                      |
| `max_entries`      | `15`                                       | Maximum number of suggestions the spell checker can return |
| `aliases`          | `[Alias(name: "en", dictionary: "en_US")]` | List of aliased names for dictionaries                     |

### Aliases

An `Alias` can be used to create shorthands for a language.
New aliases must be added to the `aliases` array.

An `Alias` has two attributes, a `name` and a `dictionary`.
The `name` attribute is the aliased name to use.
The `dictionary` attribute is the name of the actual dictionary.
This can not be another alias.

**NOTE**: The `default_language` can be an alias.
