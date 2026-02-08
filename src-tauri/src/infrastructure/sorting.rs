use std::cmp::Ordering;
use std::path::Path;

pub fn compare_entries(path_a: &Path, is_dir_a: bool, path_b: &Path, is_dir_b: bool) -> Ordering {
    match (is_dir_a, is_dir_b) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => {
            let a = path_a.to_string_lossy();
            let b = path_b.to_string_lossy();
            let primary = a.to_lowercase().cmp(&b.to_lowercase());
            if primary == Ordering::Equal {
                a.cmp(&b)
            } else {
                primary
            }
        }
    }
}
