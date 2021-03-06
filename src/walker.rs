/*
 *  author: Aleksei Kozadaev (2020)
 */

use std::fs;
use std::io;
use std::path::Path;

pub struct Walker<'a> {
    enriched: &'a mut Vec<String>,
}

fn is_ignored(path: &str) -> bool {
    // TODO: add parameter to be able to add arbitrary filters.
    let ignore = ["/.git", "/.hg", "/node_modules", "/target"];

    for (_, sfx) in ignore.iter().enumerate() {
        if path.ends_with(sfx) {
            return true;
        }
    }

    false
}

impl<'a> Walker<'a> {
    pub fn new(dest: &'a mut Vec<String>) -> Self {
        Walker { enriched: dest }
    }

    pub fn process(&mut self, files: &[String]) -> io::Result<()> {
        for (_, file) in files.iter().enumerate() {
            let path = Path::new(file);

            self.enriched.push(file.clone());

            if path.is_dir() {
                self.walk(Path::new(file))?
            }
        }

        Ok(())
    }

    fn walk(&mut self, path: &Path) -> io::Result<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry_path = entry?.path();

                if entry_path.is_dir() {
                    let str_path = entry_path.to_str().unwrap();

                    if !is_ignored(str_path) {
                        self.enriched.push(str_path.to_string());
                        self.walk(&entry_path)?;
                    }
                }
            }
        }

        Ok(())
    }
}
