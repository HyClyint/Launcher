use std::{fs, path::PathBuf};

pub fn get_jars_in_directory(directory: &PathBuf) -> Vec<PathBuf> {
    let mut jars = vec![];

    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            if path.extension().unwrap() == "jar" {
                jars.push(path);
            }
        } else {
            jars.extend(get_jars_in_directory(&path));
        }
    }

    jars
}

pub fn get_percentage(current: i32, total: i32) -> f64 {
    let current = current as f64;
    let total = total as f64;

    let percentage = (current / total) * 100.0;

    (percentage * 10.0).round() / 10.0
}
