use regex::RegexBuilder;

pub fn clean_drafts(poem: &str) -> &str {
    RegexBuilder::new("(---.*?---.*?)((---)|$)")
        .dot_matches_new_line(true)
        .build()
        .unwrap()
        .captures(poem)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
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
publish: true
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
