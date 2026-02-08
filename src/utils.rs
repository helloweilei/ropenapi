pub fn get_file_name(path: &str) -> String {
    let file_name = path.split_once('\\').unwrap_or(("", "")).1;

    file_name.split_once('.').unwrap_or(("", "")).0.to_string()
}
