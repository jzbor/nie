use colored::Colorize;

pub fn inform(s: &str) {
    eprintln!("{}", s.to_string().bright_blue())
}

pub fn announce(s: &str) {
    eprintln!("\n{}", format!("=> {}", s).bright_green())
}
