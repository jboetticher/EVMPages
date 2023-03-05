use std::{fs::{read_dir, DirEntry}, path::Path, io};

pub fn print_files() {
    let dir: &Path = Path::new(&env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    
    match visit_dirs(dir, &print_dir) {
        Ok(_) => (),
        Err(err) => println!("There was an error with visiting directories: {}", err),
    };

    // Some(dir)
}

pub fn select_html

fn print_dir(d: &DirEntry) {
    println!("{:?}", d.file_name());
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