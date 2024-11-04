use crate::Config;

#[test]
fn test_load_from_file() {
    match Config::load_from_file("../../config.toml".to_string()) {
        Ok(c) => {
            println!("{:?}", c);
        }
        Err(e) => {
            println!("Error:{e}")
        }
    }
}

#[test]
fn test_default() {
    println!("{:?}", Config::default());
}