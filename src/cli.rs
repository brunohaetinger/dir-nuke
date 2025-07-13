use std::env;

pub fn is_verbose() -> bool {
    env::args().any(|arg| arg == "--verbose")
}

pub fn get_target_path() -> String {
    env::args().nth(1).unwrap_or_else(|| ".".to_string())
}
