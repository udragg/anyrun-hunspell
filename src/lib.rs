#![warn(clippy::nursery)]
// TODO remove all functions that could panic
// panicing crashes anyrun
use abi_stable::std_types::{
    ROption::{RNone, RSome},
    RString, RVec,
};
use anyrun_plugin::*;
use serde::{Deserialize, Serialize};
use std::{
    char,
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    process::{Command, Stdio},
    str::FromStr,
};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    prefix: String,
    default_language: String,
    max_entries: u32,
    aliases: Vec<Alias>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct Alias {
    /// Aliased name of the dictionary
    name: String,
    /// Original name of the dictionary
    dictionary: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: ":spell".into(),
            default_language: String::from("en_US"),
            max_entries: 15,
            aliases: vec![Alias {
                name: "en".into(),
                dictionary: "en_US".into(),
            }],
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum HunspellCompletion {
    Correct {
        word: String,
    }, // output tarts with `*`, `+` or `-`
    NearMiss {
        word: String,
        near_misses: Vec<String>,
    }, // output starts with `&`
    Incorrect {
        word: String,
    }, // output starts with `#`
    LineEnd, // output is a blank line (end of input line)
}

impl FromStr for HunspellCompletion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.starts_with("* ") || s.starts_with("- ") => {
                let word = s
                    .strip_prefix("* ")
                    .or_else(|| s.strip_prefix("- "))
                    .ok_or(())?
                    .into();
                Ok(Self::Correct { word })
            }
            s if s.starts_with("+ ") => {
                let line = s.strip_prefix("+ ").ok_or(())?;
                let (word, _root) = line.rsplit_once(" ").ok_or(())?;
                Ok(Self::Correct { word: word.into() })
            }
            s if s.starts_with("& ") => {
                let stripped = s
                    .strip_prefix("& ")
                    .expect("Matched on string starting with prefix");
                let (info, suggestions) = stripped.split_once(": ").ok_or(())?;

                let mut info_iter = info.rsplitn(3, " ");
                let _offset = info_iter.next().ok_or(())?;
                let _count = info_iter.next().ok_or(())?;
                let original = info_iter.next().ok_or(())?;

                let near_misses = suggestions
                    .split(", ")
                    .map(|suggestion| suggestion.to_owned())
                    .collect();

                Ok(Self::NearMiss {
                    word: original.to_string(),
                    near_misses,
                })
            }
            s if s.starts_with("# ") => {
                let stripped = s
                    .strip_prefix("# ")
                    .expect("Matched on string starting with prefix");
                let (original, _offset) = stripped.split_once(" ").ok_or(())?;
                Ok(Self::Incorrect {
                    word: original.to_string(),
                })
            }
            _ => Err(()),
        }
    }
}

fn completion_to_matches(completion: HunspellCompletion) -> Option<Vec<Match>> {
    match completion {
        HunspellCompletion::Correct { word } => Some(vec![Match {
            title: word.into(),
            description: RSome("✔ Correct".into()),
            use_pango: true,
            icon: RNone,
            id: RSome(0),
        }]),
        HunspellCompletion::NearMiss { word, near_misses } => Some(
            near_misses
                .into_iter()
                .map(|near_miss| Match {
                    title: near_miss.into(),
                    description: RSome(
                        // format!("🖉 Suggestion for '<span foreground=\"#f0c6c6\">{word}</span>'")
                        format!("🖉 Suggestion for '{word}'").into(),
                    ),
                    use_pango: true,
                    icon: RNone,
                    id: RSome(1),
                })
                .collect::<Vec<_>>(),
        ),
        HunspellCompletion::Incorrect { word } => Some(vec![Match {
            title: format!("𐄂 No suggestions for '{word}'").into(),
            description: RNone,
            use_pango: true,
            icon: RNone,
            id: RSome(2),
        }]),
        HunspellCompletion::LineEnd => None,
    }
}

#[init]
fn init(config_dir: RString) -> (Config, HashSet<String>, HashMap<String, String>) {
    let cmd = Command::new("hunspell")
        .arg("-D")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped()) // hunspell -D outputs to stderr, for some reason
        .output()
        .expect("failed to run hunspell");
    let out = String::from_utf8_lossy(&cmd.stderr);
    let dicts = out
        .lines()
        .skip(3)
        .take_while(|l| l.starts_with("/"))
        .map(|l| {
            l.rsplit_once("/")
                .map(|(_, dict_name)| dict_name)
                .unwrap_or(l)
        })
        .map(|d| d.to_owned())
        .collect::<HashSet<_>>();

    let config = fs::read_to_string(format!("{config_dir}/hunspell.ron")).map_or_else(
        |_| Config::default(),
        |content| {
            ron::from_str(&content).unwrap_or_else(|err| {
                eprintln!("anyrun-hunspell: failed to parse hunspell.ron: {err:#?}");
                Default::default()
            })
        },
    );

    let aliases = config
        .aliases
        .iter()
        .map(|alias| (alias.name.to_owned(), alias.dictionary.to_owned()))
        .filter(|(_, dict)| dicts.contains(dict))
        .collect::<HashMap<_, _>>();

    (config, dicts, aliases)
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Hunspell".into(),
        icon: "accessories-character-map".into(),
    }
}

#[get_matches]
fn get_matches(
    input: RString,
    config: &(Config, HashSet<String>, HashMap<String, String>),
) -> RVec<Match> {
    let input = if let Some(query) = input.strip_prefix(&config.0.prefix) {
        query
    } else {
        return RVec::new();
    };

    let (input_lang, text, lang_ended) = input.strip_prefix(":").map_or_else(
        || {
            (
                config.0.default_language.as_str(),
                input.trim(),
                input.ends_with(char::is_whitespace),
            )
        },
        |stripped| {
            stripped
                .split_once(char::is_whitespace)
                .map_or((stripped, "", false), |(lang, text)| (lang, text, true))
        },
    );

    let lang = config.2.get(input_lang).map_or(input_lang, |l| l);

    let lang_invalid = !config.1.contains(lang);
    let text_empty = text.trim().is_empty();
    match (!lang_invalid, !text_empty, lang_ended) {
        (false, _, true) | (false, true, _) => {
            // lang invalid and lang ended (whitespace after) OR lang invalid and text non-empty => dict err
            return vec![Match {
                title: format!("Dictionary Not Found: {lang}").into(),
                description: RNone,
                use_pango: false,
                icon: RNone,
                id: RNone,
            }]
            .into();
        }
        (false, false, false) | (true, false, _) => {
            // lang invalid but unfinished OR lang valid but no text => wait
            return vec![Match {
                title: "Waiting...".into(),
                description: RNone,
                use_pango: false,
                icon: RNone,
                id: RNone,
            }]
            .into();
        }
        (true, true, _) => (), // lang is valid and text is non-empty => continue
    }

    let mut hunspell_cmd = Command::new("hunspell")
        .arg("-a") // pipe interface
        .arg("-d") // select dictionary
        .arg(lang)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run hunspell");
    let owned_text = text.to_owned();
    let mut hunspell_stdin = hunspell_cmd.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        hunspell_stdin
            .write_all(format!("`\n^{owned_text}").as_bytes())
            .expect("Failed to write to stdin")
    });
    let hunspell_output = hunspell_cmd
        .wait_with_output()
        .expect("Failed to open stdout");

    String::from_utf8_lossy(&hunspell_output.stdout)
        .lines()
        .filter_map(|line| line.parse::<HunspellCompletion>().ok())
        .filter_map(completion_to_matches)
        .flatten()
        .take(config.0.max_entries as usize)
        .collect::<Vec<_>>()
        .into()
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    if let RSome(id) = selection.id {
        match id {
            0 => HandleResult::Copy(selection.title.into_bytes()), // 0: correct
            1 => HandleResult::Copy(selection.title.into_bytes()), // 1: completion suggesiton
            2 => HandleResult::Close, // 2: Incorrect word, no suggestions
            _ => HandleResult::Close, // Catchall
        }
    } else {
        HandleResult::Close
    }
}
