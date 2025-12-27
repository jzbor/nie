use colored::Colorize;

pub fn announce(s: &str) {
    eprintln!("{}", s.to_string().bright_blue())
}
