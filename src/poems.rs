use regex::RegexBuilder;

pub fn clean_drafts(poem: &str) -> &str {
    // FIXME regex doesn't support look around...
    let regex_without_draft = RegexBuilder::new("(---.*---.*)(?=---)")
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    match regex_without_draft.captures(poem) {
        Some(captures) => captures.get(0).unwrap().as_str(),
        None => poem,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_not_change_draftless() {
        let contents = "\
---
collections: sample
publish: true
---

ducky

";
        let result = clean_drafts(contents);
        assert_eq!(contents, result);
    }

    #[test]
    fn should_remove_draft() {
        let contents = "\
---
collections: sample
publish: trueU
---

ducky

---

some beginning draft
i don't want published
";
        let expected = "\
---
collections: sample
publish: true
---

ducky

";
        let result = clean_drafts(contents);
        assert_eq!(expected, result);
    }

    #[test]
    fn should_remove_multiple_draft() {
        let contents = "\
---
collections: sample
publish: true
---

ducky

---

some beginning draft

---

i don't want published
";
        let expected = "\
---
collections: sample
publish: true
---

ducky

";
        let result = clean_drafts(contents);
        assert_eq!(expected, result);
    }
}
