use std::io::Write;

pub fn input_string(pr: &str) -> String {
    let mut p: String = String::new();
    print!("{}", pr);
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut p).unwrap();
    p.trim().to_string()
}