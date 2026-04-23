use std::io::Write;
use std::process::exit;
use std::vec;

pub fn input() -> String {
    let mut p: String = String::new();
    std::io::stdin().read_line(&mut p).unwrap();
    p.trim().to_string()
}

pub fn input_string(pr: &str) -> String {
    print!("{}", pr);
    std::io::stdout().flush().unwrap();
    input()
}

pub fn stop_work_and_output(pr: &str) {
    println!("{}", pr);
    input();
    exit(0);
}

pub fn confim_input(pr: &str) -> bool {
    let confirm_rec = input_string(&format!("{}(y, N):", pr));
    let confirm_options = ["y", "yes", "yup"];
    confirm_options.iter().any(|el| *el == confirm_rec)
}