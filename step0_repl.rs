use std::io::{self, Write};

fn main() {
    if let Err(e) = _main() {
        eprintln!("error: {}", e);
    }
}

fn _main() -> Result<(), std::io::Error> {
    loop {
        print!("user> ");
        io::stdout().flush()?;
        let mut buffer = String::new();
        if let 0 = io::stdin().read_line(&mut buffer)? {
            // If the stream reaches EOF, `read_line` returns `Ok(0)`. On EOF,
            // the program exits.
            return Ok(())
        }
        println!("{}", rep(&buffer));
    }
}

fn rep(input: &str) -> &str {
    print(eval(read(input)))
}

fn read(input: &str) -> &str { input }

fn eval(input: &str) -> &str { input }

fn print(input: &str) -> &str { input }

