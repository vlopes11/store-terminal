use std::io::{self, stdout, BufRead, Error, ErrorKind, Lines, StdinLock, Write};
use std::str::SplitWhitespace;
use store_terminal::prelude::*;

const NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");
const VERSION_MAJOR: Option<&'static str> = option_env!("CARGO_PKG_VERSION_MAJOR");
const VERSION_MINOR: Option<&'static str> = option_env!("CARGO_PKG_VERSION_MINOR");
const AUTHORS: Option<&'static str> = option_env!("CARGO_PKG_AUTHORS");

macro_rules! fetch_text {
    ($x:expr) => {
        $x.unwrap_or("(undefined)")
    };
}

enum State {
    Executing,
    ShouldFinish,
}

fn main() {
    println!(
        "{} v{}.{} by [{}]",
        fetch_text!(NAME),
        fetch_text!(VERSION_MAJOR),
        fetch_text!(VERSION_MINOR),
        fetch_text!(AUTHORS),
    );

    print!("Initializing...");
    let terminal = Terminal::new().unwrap();
    terminal.init().unwrap();
    println!("terminal initialized!");

    print_help();

    let stdin = io::stdin();

    let mut iterator = stdin.lock().lines();
    let mut state = State::Executing;
    while let State::Executing = state {
        state = if let Some(line) = get_line(&mut iterator) {
            proc_command(line, &terminal).unwrap_or_else(|e| {
                println!("Error: {:?}", e);
                state
            })
        } else {
            State::ShouldFinish
        };
    }

    println!("Bye!");
}

fn get_line(iterator: &mut Lines<StdinLock>) -> Option<String> {
    print!("> ");
    if stdout().flush().is_err() {
        return None;
    }
    iterator
        .next()
        .unwrap_or(Err(Error::new(ErrorKind::Other, "No input provided")))
        .map(|l| Some(l.trim().to_owned()))
        .unwrap_or(None)
}

fn proc_command(line: String, terminal: &Terminal) -> Result<State, ErrorVariant> {
    let mut iter = line.split_whitespace();

    match iter.next() {
        Some(c) if c.to_lowercase() == "q" => return Ok(State::ShouldFinish),
        Some(c) if c.to_lowercase() == "h" => print_help(),
        Some(c) if c.to_lowercase() == "cart" => return proc_command_cart(iter, terminal),
        Some(c) if c.to_lowercase() == "c" => return proc_command_cart(iter, terminal),
        Some(c) if c.to_lowercase() == "db" => println!("{}", terminal.get_db()?),
        None => (),
        _ => {
            println!("Command `{}` not recognized!", line);
            print_help();
        }
    }

    Ok(State::Executing)
}

fn proc_command_cart(
    mut iter: SplitWhitespace,
    terminal: &Terminal,
) -> Result<State, ErrorVariant> {
    match iter.next() {
        Some(c) if c.to_lowercase() == "print" => println!("{}", terminal.get_cart()?),
        Some(c) if c.to_lowercase() == "p" => println!("{}", terminal.get_cart()?),
        Some(c) if c.to_lowercase() == "reset" => println!("{:?}", terminal.reset_cart()?),
        Some(c) if c.to_lowercase() == "r" => println!("{:?}", terminal.reset_cart()?),
        Some(c) if c.to_lowercase() == "scan" => return proc_command_cart_scan(iter, terminal),
        Some(c) if c.to_lowercase() == "s" => return proc_command_cart_scan(iter, terminal),
        Some(c) => {
            println!("Cart command `{}` not recognized!", c);
            print_help();
        }
        None => {
            println!("Cart command not provided!");
            print_help();
        }
    }

    Ok(State::Executing)
}

fn proc_command_cart_scan(
    mut iter: SplitWhitespace,
    terminal: &Terminal,
) -> Result<State, ErrorVariant> {
    match iter.next() {
        Some(c) => terminal.scan(c.to_string())?,
        None => {
            println!("Code not provided!");
            print_help();
        }
    }

    Ok(State::Executing)
}

fn print_help() {
    println!("Available commands:");
    println!("&cart &print\t\tPrint the current contents");
    println!("&cart &reset\t\tReset the contents");
    println!("&cart &scan [code]\tScan the given set of codes");
    println!("db\t\t\tPrint the database contents");
    println!("h\t\t\tShow this menu");
    println!("q\t\t\tQuit");
}
