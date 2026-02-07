#[macro_export]
macro_rules! navkey {
    (down) => { KeyCode::Char('j') | KeyCode::Down };
    (up) => { KeyCode::Char('k') | KeyCode::Up };
    (left) => { KeyCode::Char('h') | KeyCode::Left };
    (right) => { KeyCode::Char('l') | KeyCode::Right };
}
