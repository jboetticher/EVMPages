use std::{fs::{read_dir, DirEntry}, path::{Path, PathBuf}, io, ffi::OsStr};
use inquire::Select;

pub fn select_html(dir: &Path) -> io::Result<PathBuf> {
    let mut options: Vec<String>;
    let mut paths: Vec<PathBuf>;

    // Initialize options
    match dir.parent() {
        Some(parent) => {
            options = vec!["▲ Parent Directory".to_string()];
            paths = vec![parent.to_path_buf()];
        },
        None => {
            options = Vec::new();
            paths = Vec::new();
        }
    }

    // Select paths
    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            match path.to_str() {
                Some(name) => {
                    let path_is_dir: bool = path.is_dir();
                    if path.extension().unwrap_or(OsStr::new("")) == "html"|| path_is_dir {
                        options.push(name.to_string());
                        paths.push(path);
                    }
                },
                None => {
                    continue;
                }
            }
        }
    }

    // Select
    let selection_prompt = Select::new(
        "Select an HTML file:",
        options,
    );

    // Either recursive or select HTML
    match selection_prompt.raw_prompt() {
        Ok(selection) => {
            let chosen = &paths[selection.index];
            if chosen.is_dir() {
                return select_html(&chosen);
            }

            Ok(chosen.clone())
        },
        Err(e) => {
            panic!("{:?}", e);
        }
    }
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            cb(&entry);
        }
    }
    Ok(())
}