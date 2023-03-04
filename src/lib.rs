use std::collections::HashMap;
use std::error::Error;
use std::fs;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.path)?;
    let tags = get_tags(&contents);
    println!("{:#?}", tags);

    // get all files
    // get tags for all files
    // filter out unpublished files
    // create collections from published files

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

fn is_published(tags: HashMap<&str, &str>) -> bool {
    let r = match tags.get("publish") {
        Some(v) => *v == "true",
        None => false,
    };
    r
}

fn trace<T: core::fmt::Debug + core::fmt::Display>(x: T) -> T {
    println!("{:#?}", x);
    x
}

fn get_tags(contents: &str) -> Result<HashMap<&str, &str>, &str> {
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
    trace(tags);

    let lines = tags
        .trim()
        .lines()
        .map(|l| l.split(":").map(|v| v.trim()).collect::<Vec<&str>>())
        // TODO this will produce unintuitive error messages
        .map(|a| (*a.get(0).unwrap(), *a.get(1).unwrap()))
        .collect::<HashMap<&str, &str>>();

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

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
