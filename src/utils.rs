pub fn get_file_name(path: &str) -> String {
    let file_name = path.rsplit_once('\\').unwrap_or(("", "")).1;

    file_name.rsplit_once('.').unwrap_or(("", "")).0.to_string()
}

#[test]
fn test_get_file_name() {
    assert_eq!(get_file_name("C:\\Users\\user\\Desktop\\file.txt"), "file");
}
