use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::hash::Hash;
use std::io;
use std::path;
use walkdir::WalkDir;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // TODO cleanup
    let path = path::Path::new(&config.path);
    let mut collection_aggregator: HashMap<&str, Vec<&str>> = HashMap::new();

    let collections: HashMap<String, String> = WalkDir::new(config.path)
        .into_iter()
        .filter_map(|f| {
            let a = f.unwrap();
            // TODO skip directories
            let r = fs::read_to_string(a.path()).unwrap();
            let t = get_tags(&r).unwrap();
            if is_published(&t) {
                return Some(t);
            } else {
                return None;
            }
        })
        .flatten() // huh, not sure why I need this atm, was it nested?
        .collect();

    dbg!(collections);

    // visit_dirs(path, &|entry: &fs::DirEntry| {
    //     let path = entry.path();

    //     let contents = fs::read_to_string(path.clone()).unwrap();
    //     let tags = get_tags(&contents).unwrap();
    //     if is_published(&tags) {
    //         let collections = tags.get("collections");
    //         match collections {
    //             Some(c) => {
    //                 let parsed_collections = parse_collections(c);
    //                 let copy_shit = collection_aggregator.clone();

    //                 collection_aggregator =
    //                     aggregate_collections("horseshit", parsed_collections, copy_shit);
    //                 ()
    //             }
    //             None => (),
    //         };
    //     }
    // });

    // get all files
    // get tags for all files
    // filter out unpublished files
    // create collections from published files
    // write collections to new files

    Ok(())
}

fn parse_collections(raw: &str) -> Vec<&str> {
    raw.trim().split(' ').collect()
}

// taken from std::fs::read_dir docs
fn visit_dirs(dir: &path::Path, cb: &dyn Fn(&fs::DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

pub struct Config {
    pub dryrun: bool,
    pub path: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // remove filename arg
        let path = match args.next() {
            Some(q) => q,
            None => return Err("Path not specfied"),
        };

        // TODO Read dryrun from args somehow
        Ok(Config { path, dryrun: true })
    }
}

fn is_published(tags: &HashMap<String, String>) -> bool {
    let r = match tags.get("publish") {
        Some(v) => *v == "true",
        None => false,
    };
    r
}

fn get_tags(contents: &str) -> Result<HashMap<String, String>, &'static str> {
    let separator = "---";
    let mut tag_indicators = contents.match_indices(separator);
    let tag_start = match tag_indicators.next() {
        Some(i) => i.0,
        None => return Err("No tag indicators"),
    };
    if tag_start != 0 {
        return Err("Tag start was not at the beginning of the file");
    }
    let tag_end = match tag_indicators.next() {
        Some(i) => i.0,
        None => return Err("No end indicator for tags"),
    };

    let tags = match contents.get(tag_start + separator.bytes().len()..tag_end) {
        Some(s) => s,
        None => return Err("No tags"),
    };

    let lines = tags
        .trim()
        .lines()
        .map(|l| l.split(":").map(|v| v.trim()).collect::<Vec<&str>>())
        // TODO this will produce unintuitive error messages
        .map(|a| (a.get(0).unwrap().to_string(), a.get(1).unwrap().to_string()))
        .collect::<HashMap<String, String>>();

    Ok(lines)
}

fn aggregate_collections<'a, 'b>(
    name: &'a str,
    collections: Vec<&'b str>,
    mut aggregate: HashMap<&'b str, Vec<&'a str>>,
) -> HashMap<&'b str, Vec<&'a str>> {
    for c in collections {
        let mut collection_to_update: Vec<&str> = match aggregate.get(c) {
            Some(existing) => existing.clone(),
            None => {
                let something: Vec<&str> = Vec::new();
                something
            }
        };
        collection_to_update.push(name);
        aggregate.insert(c, collection_to_update);
    }
    aggregate
}
#[cfg(test)]
mod tests {
    use super::*;
    mod aggregate_collections {
        use super::*;
        #[test]
        fn create_new_aggregate() {
            let file_name_1 = "goat";
            let file_collections_1 = vec!["animal"];
            let result = aggregate_collections(file_name_1, file_collections_1, HashMap::new());
            assert_eq!(result.get("animal").unwrap()[0], "goat");
        }

        #[test]
        fn add_to_aggregate() {
            let file_name_1 = "goat";
            let file_collections_1 = vec!["animal"];
            let result = aggregate_collections(file_name_1, file_collections_1, HashMap::new());

            let file_name_2 = "horse";
            let file_collections_2 = vec!["animal"];
            let result_2 = aggregate_collections(file_name_2, file_collections_2, result);
            assert_eq!(result_2.get("animal").unwrap().len(), 2);
            assert_eq!(result_2.get("animal").unwrap().contains(&"horse"), true);
            assert_eq!(result_2.get("animal").unwrap().contains(&"goat"), true);
        }

        #[test]
        fn add_multi_collection_to_aggregate() {
            let file_name_1 = "goat";
            let file_collections_1 = vec!["animal", "pet", "horned"];
            let result = aggregate_collections(file_name_1, file_collections_1, HashMap::new());
            assert_eq!(result.get("animal").unwrap()[0], "goat");
            assert_eq!(result.get("pet").unwrap()[0], "goat");
            assert_eq!(result.get("horned").unwrap()[0], "goat");
        }
    }

    mod get_tags {
        use super::*;
        #[test]
        fn one_tag() {
            let contents = "\
---
tag: duck
---
";
            let result = get_tags(contents).unwrap();
            assert_eq!(*result.get("tag").unwrap(), "duck");
        }

        #[test]
        fn two_tags() {
            let contents = "\
---
tag: duck
tag2: rabbit
---
";
            let result = get_tags(contents).unwrap();
            assert_eq!(*result.get("tag").unwrap(), "duck");
            assert_eq!(*result.get("tag2").unwrap(), "rabbit");
        }

        #[test]
        fn missing_start_tag() {
            let contents = "\
tag: duck
---
";
            let result = get_tags(contents).err().unwrap();
            assert_eq!(result, "Tag start was not at the beginning of the file");
        }

        #[test]
        fn missing_end_tag() {
            let contents = "\
---
tag: duck
bloopy bolp
";
            let result = get_tags(contents).err().unwrap();
            assert_eq!(result, "No end indicator for tags");
        }

        #[test]
        fn multi_value_tags() {
            let contents = "\
---
tag: duck goat sheep chicken
tag2: rabbit
---
";
            let result = get_tags(contents).unwrap();
            assert_eq!(*result.get("tag").unwrap(), "duck goat sheep chicken");
            assert_eq!(*result.get("tag2").unwrap(), "rabbit");
        }
    }
}
