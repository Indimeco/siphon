use std::error::Error;
use std::fs;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.path)?;
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

fn search<'a>(query: &str, contents: &'a str, insensitive: bool) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| {
            if insensitive {
                line.to_lowercase().contains(&query.to_lowercase())
            } else {
                line.contains(query)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    mod search_test {
        use super::*;
        #[test]
        fn one_result() {
            let query = "duct";
            let contents = "\
Rust:
safe, fast, productive.
Pick three.";

            assert_eq!(
                vec!["safe, fast, productive."],
                search(query, contents, false)
            );
        }

        #[test]
        fn case_insensitive() {
            let query = "rUsT";
            let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

            assert_eq!(vec!["Rust:", "Trust me."], search(query, contents, true));
        }
    }
}
