use rustyline::Editor;
use rustyline::error::ReadlineError;

fn main() {
    // Construct a new `Editor` without a syntax specific helper. This means
    // that there is no tokenizer/parser used for completion, suggestion, or
    // highlighting.
    let mut rl = Editor::<()>::new();
    if rl.load_history(".mal_history").is_err() {
        println!("no previous history");
    }
    loop {
        match rl.readline("user> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("{}", rep(&line));
            },
            Err(ReadlineError::Eof) => {
                // If the stream reaches EOF, we exit the program.
                return
            },
            Err(err) => {
                eprintln!("error: {}", err);
            },
        }
        rl.save_history(".mal_history").expect("error saving history");
    }
}

fn rep(input: &str) -> &str {
    print(eval(read(input)))
}

fn read(input: &str) -> &str { input }

fn eval(input: &str) -> &str { input }

fn print(input: &str) -> &str { input }

