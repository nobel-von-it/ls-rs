#[test]
fn terminal_size_validation_test() {
    let res = ls_rs::term::terminal_size();
    assert!(res.is_some());
    let (cols, rows) = res.unwrap();
    assert!(cols > 0);
    assert!(rows > 0);
}
