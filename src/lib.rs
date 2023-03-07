use alloc::collections;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use walkdir::WalkDir;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let collections: HashMap<String, Vec<String>> = WalkDir::new(config.path)
        .into_iter()
        .filter_map(|f| {
            let a = f.unwrap();
            if a.metadata().unwrap().is_dir() {
                return None;
            }
            let r = fs::read_to_string(a.path()).unwrap();
            let t = get_tags(&r).unwrap();
            // skip unpublished poems
            if is_published(&t) {
                let collections = parse_collections(t.get("collections").unwrap());
                return Some((
                    a.file_name().to_str().unwrap().to_string(),
                    collections.clone(),
                ));
            } else {
                return None;
            }
        })
        .fold(
            HashMap::new(),
            |acc: HashMap<String, Vec<String>>, (name, collections)| {
                aggregate_collections(name, collections, acc)
            },
        );

    for collection in collections {
        // look for existing collection files
        // read collection files for title, desc, etc
        // create CollectionData struct, default missing collection files values
        // create_collection_template
        // write file to where? needs a target dir I guess
    }

    dbg!(collections);

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

fn parse_collections(raw: &str) -> Vec<String> {
    raw.trim().split(' ').map(|a| a.to_string()).collect()
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
    name: String,
    collections: Vec<String>,
    mut aggregate: HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    for c in collections {
        let mut collection_to_update: Vec<String> = match aggregate.get(&c) {
            Some(existing) => existing.clone(),
            None => {
                let something: Vec<String> = Vec::new();
                something
            }
        };
        collection_to_update.push(name.clone());
        aggregate.insert(c, collection_to_update);
    }
    aggregate
}

struct CollectionData {
    title: String,
    created: String,
    poems: Vec<String>,
    desc: String,
}

fn create_collection_template(data: CollectionData) -> String {
    let CollectionData {
        title,
        created,
        poems,
        desc,
    } = data;
    let formatted_poems = poems
        .iter()
        .map(|f| format!("- {f}"))
        .collect::<Vec<String>>()
        .join("\n");
    format!(
        "---
title: {title}
created: {created}
poems:
{formatted_poems}
---
{desc}
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    mod aggregate_collections {
        use super::*;
        #[test]
        fn create_new_aggregate() {
            let file_name_1 = String::from("goat");
            let file_collections_1 = vec![String::from("animal")];
            let result = aggregate_collections(file_name_1, file_collections_1, HashMap::new());
            assert_eq!(result.get("animal").unwrap()[0], "goat");
        }

        #[test]
        fn add_to_aggregate() {
            let file_name_1 = String::from("goat");
            let file_collections_1 = vec![String::from("animal")];
            let result = aggregate_collections(file_name_1, file_collections_1, HashMap::new());

            let file_name_2 = String::from("horse");
            let file_collections_2 = vec![String::from("animal")];
            let result_2 = aggregate_collections(file_name_2, file_collections_2, result);
            assert_eq!(result_2.get("animal").unwrap().len(), 2);
            assert_eq!(
                result_2
                    .get("animal")
                    .unwrap()
                    .contains(&"goat".to_string()),
                true
            );
            assert_eq!(
                result_2
                    .get("animal")
                    .unwrap()
                    .contains(&"horse".to_string()),
                true
            );
        }

        #[test]
        fn add_multi_collection_to_aggregate() {
            let file_name_1 = String::from("goat");
            let file_collections_1 = vec![
                String::from("animal"),
                String::from("pet"),
                String::from("horned"),
            ];
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
    mod template_collection {
        use super::*;

        #[test]
        fn creates_template_from_collection() {
            let collection = CollectionData {
                title: String::from("collection title"),
                created: String::from("2023-03-08"),
                poems: vec![String::from("name1")],
                desc: String::from("A description of the contents"),
            };
            let expected = "\
---
title: collection title
created: 2023-03-08
poems:
- name1
---
A description of the contents
";
            assert_eq!(create_collection_template(collection), expected);
        }
    }
}
