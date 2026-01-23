#[macro_export]
macro_rules! dbg_file {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .unwrap();
        writeln!(f, $($arg)*).unwrap();
    }};
}
