use glob::glob;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

mod collections;

// TODO clean draft poems

pub struct Config {
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

        Ok(Config { path, target_dir })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let source_dir = &config.path;
    let target_dir = &config.target_dir;
    let md_pattern = format!("{source_dir}/**/*.md");
    let target_collections_dir = format!("{target_dir}/collections/");
    let target_poems_dir = format!("{target_dir}/poems/");

    // clean existing dirs
    fs::remove_dir_all(&target_collections_dir).unwrap_or(());
    fs::remove_dir_all(&target_poems_dir).unwrap_or(());
    fs::create_dir(&target_collections_dir).expect("Cannot create collections dir");
    fs::create_dir(&target_poems_dir).expect("Cannot create poems dir");

    let collections = glob(&md_pattern)
        .expect("Failed to read glob pattern")
        .filter_map(|glob_result| {
            let path_buf = glob_result.unwrap();
            if path_buf.metadata().unwrap().is_dir() {
                return None;
            }
            let file_name = match path_buf.file_name().unwrap().to_str() {
                Some(n) => n.strip_suffix(".md").unwrap().to_string(),
                None => return None,
            };
            let file_contents = match fs::read_to_string(path_buf.as_path()) {
                Ok(contents) => contents,
                Err(e) => {
                    eprintln!("Failed to read file: {file_name}, {e}");
                    return None;
                }
            };
            let t = match collections::get_tags(&file_contents) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to read tags for {file_name}, {e}");
                    return None;
                } // skip stuff that doesn't have tags
            };
            // skip unpublished poems
            if collections::is_published(&t) {
                let collections = match t.get("collections") {
                    Some(c) => collections::parse_collections(c),
                    None => return None, // no collections, we leave now
                };
                let target_path = format!("{target_poems_dir}/{file_name}.md");
                // lol side effect inside a map, should move this out
                fs::write(&target_path, file_contents).unwrap();
                println!("Wrote poem: {target_path}");
                return Some((file_name, collections.clone()));
            } else {
                return None;
            }
        })
        .fold(
            HashMap::new(),
            |acc: HashMap<String, Vec<String>>, (name, collections)| {
                collections::aggregate_collections(name, collections, acc)
            },
        );

    let collection_data = collections.iter().map(|(collection_name, poems)| {
        let path = format!("{source_dir}/**/{collection_name}.md");
        let existing_collections = glob(&path)
            .expect("Failed to read glob pattern")
            .map(|x| x.unwrap())
            .collect::<Vec<PathBuf>>();
        if existing_collections.len() > 1 {
            panic!("More than one collection named {collection_name}");
        }
        if existing_collections.len() < 1 {
            panic!("Did not find collection named {collection_name}");
        }
        let existing_contents = fs::read_to_string(&existing_collections[0]).unwrap();
        let parsed_collection = collections::parse_collection_template(&existing_contents);
        let updated_collection =
            collections::update_collection_poems(parsed_collection, poems.clone());
        return (collection_name, updated_collection);
    });

    collection_data.for_each(|(collection_name, collection_data)| {
        let template = collections::create_collection_template(collection_data);
        let mut target_path = PathBuf::from(&target_collections_dir).join(collection_name);
        target_path.set_extension("md");
        fs::write(&target_path, template).unwrap();
        let target_path_str = target_path.to_str().unwrap();
        println!("Wrote collection: {target_path_str}");
    });

    Ok(())
}
