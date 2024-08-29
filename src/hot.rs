use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
};

use notify::Watcher;

#[macro_export]
macro_rules! hot_const_str {
    ($id: ident, $value: literal) => {
        pub fn $id() -> &'static str {
            static INNER: ::std::sync::RwLock<&'static str> = ::std::sync::RwLock::new($value);

            #[$crate::hot_distributed_slice($crate::hot::HOT_CONSTANTS)]
            static MY_VALUE_INSTANCE: $crate::hot::MutableConstInstance =
                $crate::hot::MutableConstInstance {
                    name: stringify!($id),
                    read_value: &|| $crate::hot::unescape_and_quote(&INNER.read().unwrap()),
                    setter: &|s| $crate::hot::try_set_string(s, &INNER),
                };

            *INNER.read().unwrap()
        }
    };
}

pub fn try_set_string(
    s: String,
    rw_lock: &std::sync::RwLock<&'static str>,
) -> Result<bool, String> {
    let current_value = *rw_lock.read().unwrap();

    let s = s
        .strip_prefix("\"")
        .ok_or_else(|| "String does not begin with `\"`")?;
    let s = s
        .strip_suffix("\"")
        .ok_or_else(|| "String does not end with `\"`")?;

    let escaped = escape(&s);

    if current_value == escaped.as_str() {
        return Ok(false);
    }

    let mut w = rw_lock.write().unwrap();

    *w = String::leak(escaped);

    Ok(true)
}

pub fn unescape_and_quote(s: &str) -> String {
    let mut s = s.to_string();
    s = s
        .replace("\t", "\\t")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\\", "\\");

    format!("\"{s}\"")
}

pub fn escape(s: &str) -> String {
    let mut s = s.to_string();
    s = s
        .replace(r#"\t"#, "\t")
        .replace(r#"\n"#, "\n")
        .replace(r#"\r"#, "\r")
        .replace(r#"\"#, "\\");

    s
}

#[macro_export]
macro_rules! hot_const {
    ($id: ident, $ty: ty, $value: expr) => {
        pub fn $id() -> $ty {
            static INNER: ::std::sync::RwLock<$ty> = ::std::sync::RwLock::new($value);

            #[$crate::hot_distributed_slice($crate::hot::HOT_CONSTANTS)]
            static MY_VALUE_INSTANCE: $crate::hot::MutableConstInstance =
                $crate::hot::MutableConstInstance {
                    name: stringify!($id),
                    read_value: &|| INNER.read().unwrap().to_string(),
                    setter: &|s| match <$ty as ::core::str::FromStr>::from_str(s.as_str()) {
                        Ok(new_value) => {
                            let current_value = *INNER.read().unwrap();
                            if current_value == new_value {
                                return Ok(false);
                            }

                            let mut w = INNER.write().unwrap();

                            *w = new_value;

                            Ok(true)
                        }
                        Err(err) => Err(err.to_string()),
                    },
                };

            *INNER.read().unwrap()
        }
    };

    ($id: ident, $ty: ty, $value: expr, $to_str:expr, $from_str:expr) => {
        pub fn $id() -> $ty {
            static INNER: ::std::sync::RwLock<$ty> = ::std::sync::RwLock::new($value);

            #[$crate::hot_distributed_slice($crate::hot::HOT_CONSTANTS)]
            static MY_VALUE_INSTANCE: $crate::hot::MutableConstInstance =
                $crate::hot::MutableConstInstance {
                    name: stringify!($id),
                    read_value: &|| $to_str(&INNER.read().unwrap()),
                    setter: &|s| match $from_str(s.as_str()) {
                        Ok(new_value) => {
                            let current_value = *INNER.read().unwrap();
                            if current_value == new_value {
                                return Ok(false);
                            }

                            let mut w = INNER.write().unwrap();

                            *w = new_value;

                            Ok(true)
                        }
                        Err(err) => Err(err.to_string()),
                    },
                };

            *INNER.read().unwrap()
        }
    };
}

#[linkme::distributed_slice]
pub static HOT_CONSTANTS: [MutableConstInstance];

const FILE_PATH: &'static str = "hot_constants.tsv";

#[cfg(feature = "hot")]
pub fn watch_constants(on_changed: impl Fn() + Sync + Send + 'static) {
    let constants: BTreeMap<&'static str, MutableConstInstance> =
        HOT_CONSTANTS.iter().map(|x| (x.name, x.clone())).collect();

    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .append(true)
        .create(true)
        .open(FILE_PATH)
        .expect("Could not open or create hot_constants.tsv");
    let mut file_text = String::new();
    file.read_to_string(&mut file_text)
        .expect("Could not read hot_constants.tsv");
    let unset_constants = update_constants_from_file_text1(&file_text, &constants);
    if !unset_constants.is_empty() {
        update_file_contents(&mut file, unset_constants, &constants)
    }

    drop(file);

    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => {
                //println!("Watcher event {event:?}");
                if matches!(
                    event.kind,
                    notify::EventKind::Modify(
                        notify::event::ModifyKind::Data(_) | notify::event::ModifyKind::Any
                    )
                ) {
                    match std::fs::read_to_string(FILE_PATH) {
                        Ok(text) => {
                            let changed =
                                update_constants_from_file_text2(text.as_str(), &constants);
                            if changed {
                                on_changed();
                            }
                        }
                        Err(err) => {
                            println!("file read error: {:?}", err)
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })
        .expect("Could not set up file watcher");

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch(Path::new(FILE_PATH), notify::RecursiveMode::NonRecursive)
        .expect("Could not watch file");

    Box::leak(Box::new(watcher));
}

fn update_file_contents(
    file: &mut File,
    unset_constants: BTreeSet<&'static str>,
    constants: &BTreeMap<&'static str, MutableConstInstance>,
) {
    for c in unset_constants.iter() {
        if let Some(c) = constants.get(c) {
            use std::io::Write;
            file.write_fmt(format_args!("\n{}\t{}", c.name, (*c.read_value)()))
                .unwrap();
        }
    }
}

fn update_constants_from_file_text1(
    text: &str,
    constants: &BTreeMap<&'static str, MutableConstInstance>,
) -> BTreeSet<&'static str> {
    let mut unset_constants: BTreeSet<&'static str> = constants.keys().copied().collect();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        };
        let Some((key, value)) = line.split_once('\t') else {
            println!("Could not parse hot constant line `{line}`");
            continue;
        };

        if let Some(instance) = constants.get(key) {
            if unset_constants.remove(key) {
                match (*instance.setter)(value.to_string()) {
                    Ok(changed) => {
                        if changed {
                            println!("Set `{key}` to value `{value}`")
                        }
                    }
                    Err(err) => {
                        println!("Could not set `{key}` to value `{value}`: {err}")
                    }
                }
            } else {
                println!("Constant `{key}` is defined twice");
            }
        }
    }
    unset_constants
}
fn update_constants_from_file_text2(
    text: &str,
    constants: &BTreeMap<&'static str, MutableConstInstance>,
) -> bool {
    let mut has_changed = false;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        };
        let Some((key, value)) = line.split_once('\t') else {
            println!("Could not parse hot constant line `{line}`");
            continue;
        };

        if let Some(instance) = constants.get(key) {
            match (*instance.setter)(value.to_string()) {
                Ok(changed) => {
                    if changed {
                        println!("Set `{key}` to value `{value}`");
                        has_changed = true;
                    }
                }
                Err(err) => {
                    println!("Could not set `{key}` to value `{value}`: {err}")
                }
            }
        }
    }
    has_changed
}

#[derive(Clone)]
pub struct MutableConstInstance {
    pub name: &'static str,
    pub read_value: &'static (dyn Fn() -> String + Send + Sync + 'static),
    pub setter: &'static (dyn Fn(String) -> Result<bool, String> + Send + Sync + 'static),
}
