use super::get_remote_list;

#[test]
fn test_conan_remote_list() {
    let conan_remote_list = get_remote_list();
    if let Ok(conan_remote_list) = conan_remote_list {
        assert!(conan_remote_list.len() > 0);
    }
}
