use std::{fs, path::PathBuf};
use tempfile::tempdir;

#[test]
fn test_write_collection() {
    let target_dir = tempdir().unwrap();

    let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one_poem");

    let test_config = siphon::Config {
        target_dir: String::from(target_dir.path().to_str().unwrap()),
        path: String::from(test_path.to_str().unwrap()),
    };
    let expected_output = "\
---
title: sample collection
created: 2021-06-20
poems:
- 2021-05-30
---

a description of the sample
";
    siphon::run(test_config).expect("Run failed");
    let file_output = fs::read_to_string(target_dir.path().join("collections/sample.md"))
        .expect("Collection output file should exist");
    assert_eq!(expected_output, file_output);
    let original_poem =
        fs::read_to_string(test_path.join("2021-05-30.md")).expect("Original poem should exist");
    let copied_poem = fs::read_to_string(target_dir.path().join("poems/2021-05-30.md"))
        .expect("Copied poem should exist");
    assert_eq!(copied_poem, original_poem);
}
