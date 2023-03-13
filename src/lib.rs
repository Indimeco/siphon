use regex::Regex;
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

    // let existing_collections = WalkDir::new(config.target_dir);
    // let merged_collections = collections.map(|c|{
    //     let f = existing_collections.get(c); // need by name
    //     let file_name = f.unwrap().file_name().to_str().unwrap();
    //     let merge = match collections.get(file_name) {
    //         Some(collection) => ...,
    //         None => None,
    //     }
    // })
    // for collection in collections {
    //     // look for existing collection files
    //     // read collection files for title, desc, etc
    //     // create CollectionData struct, default missing collection files values
    //     // create_collection_template
    //     // write file to where? needs a target dir I guess
    // }

    dbg!(collections);

    Ok(())
}

pub struct Config {
    pub dryrun: bool,
    pub path: String,
    pub target_dir: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // remove filename arg
        let path = match args.next() {
            Some(q) => q,
            None => return Err("Path not specfied"),
        };

        let target_dir = match args.next() {
            Some(q) => q,
            None => return Err("Target dir not specified"),
        };

        // TODO Read dryrun from args somehow
        Ok(Config {
            path,
            dryrun: true,
            target_dir,
        })
    }
}

fn update_collection_poems(collection: CollectionData, poems: Vec<String>) -> CollectionData {
    CollectionData {
        poems,
        ..collection
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

// aka crappy yaml parser
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
        .fold(vec![], |mut acc: Vec<(String, String)>, cur| {
            let before_colon = cur.get(0).unwrap();
            let list_indicator = "- ";
            // lmao this is awful
            if before_colon.starts_with(list_indicator) {
                let last_value = acc.pop().unwrap();
                let trimmed_list_value: &str = before_colon
                    .split(list_indicator)
                    .collect::<Vec<&str>>()
                    .get(1)
                    .unwrap();

                // a traumatic description of pain
                if last_value.1 == "" {
                    acc.push((last_value.0, last_value.1 + trimmed_list_value));
                } else {
                    acc.push((last_value.0, last_value.1 + ", " + trimmed_list_value));
                }
                return acc;
            } else {
                // push a new acc item
                let after_colon = cur.get(1).unwrap().to_string();
                acc.push((String::from(*before_colon), String::from(after_colon)));
                return acc;
            }
        })
        .into_iter()
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

#[derive(Debug, PartialEq)]
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

fn parse_collection_template(raw: &str) -> CollectionData {
    let tags = get_tags(raw).unwrap();
    let desc: &str = Regex::new("---.*---").unwrap().replace_all(raw); // everything except tags
    CollectionData {
        title: String::from(tags.get("title").unwrap()),
        created: String::from(tags.get("created").unwrap()),
        poems: tags
            .get("poems")
            .map(|x| parse_collections(x))
            .unwrap_or(vec![]),
        desc: String::from(desc),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod update_collection_poems {
        use super::*;
        #[test]
        fn adds_poems_to_collection() {
            let col: CollectionData = CollectionData {
                title: String::from("collection title"),
                created: String::from("2023-03-08"),
                poems: vec![String::from("name1")],
                desc: String::from("A description of the contents"),
            };
            let poems: Vec<String> = vec![String::from("name1"), String::from("name2")];
            let expected: CollectionData = CollectionData {
                title: String::from("collection title"),
                created: String::from("2023-03-08"),
                poems: vec![String::from("name1"), String::from("name2")],
                desc: String::from("A description of the contents"),
            };
            assert_eq!(update_collection_poems(col, poems), expected);
        }
    }

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

        #[test]
        fn list_tags() {
            let contents = "\
---
tag:
- list1
- list2
---
";
            let result = get_tags(contents).unwrap();
            assert_eq!(*result.get("tag").unwrap(), "list1, list2");
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

    mod parse_collection_template {
        use super::*;

        #[test]
        fn parses_partial_collection() {
            let template = "\
---
title: collection title
created: 2023-03-08
---
";
            let expected = CollectionData {
                title: String::from("collection title"),
                created: String::from("2023-03-08"),
                poems: vec![],
                desc: String::from(""),
            };
            assert_eq!(parse_collection_template(template), expected);
        }

        #[test]
        fn parses_full_template() {
            let template = "\
---
title: collection title
created: 2023-03-08
poems:
- name1
---
A description of the contents
";
            let expected = CollectionData {
                title: String::from("collection title"),
                created: String::from("2023-03-08"),
                poems: vec![String::from("name1")],
                desc: String::from("A description of the contents"),
            };
            assert_eq!(parse_collection_template(template), expected);
        }
    }
}
