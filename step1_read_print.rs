#[macro_use]
extern crate lazy_static;

use std::{fmt, iter, mem};

use regex::Regex;
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
        // Main loop prints a prompt, gets a line of input from the user,
        // calls `rep` with that line of input, and then prints out the result
        // from `rep`. It also exits when you send it an EOF (`^D` in our
        // case).
        match rl.readline("user> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match rep(&line) {
                    Ok(output) => println!("{}", output),
                    Err(err) => println!("{}", err),
                }
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

fn rep(input: &str) -> Result<String, MalError> {
    Ok(print(&eval(&read(input)?)))
}

fn read(input: &str) -> Result<MalType, MalError> {
    read_str(input)
}

fn eval(input: &MalType) -> MalType { (*input).clone() }

fn print(input: &MalType) -> String {
    pr_str(&input)
}

/// Stateful `Reader` struct that holds functions related to the reader.
///
/// Stores tokens and a position.
struct Reader<I> {
    head: Option<Token>,
    tail: I,
}

impl<I> Reader<I>
    where I: Iterator<Item=Token>,
{
    fn new(mut tokens: I) -> Self {
        let head = tokens.next();
        Self {
            head,
            tail: tokens,
        }
    }
}

impl<I> Reader<I>
    where I: Iterator<Item=Token>,
{
    /// Returns the token at the current position and increments the position.
    fn next(&mut self) -> Option<Token> {
        mem::replace(&mut self.head, self.tail.next())
    }

    /// Returns the token at the current position.
    fn peek(&self) -> Option<&Token> {
        self.head.as_ref()
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
struct Token(String);

/// Calls `tokenize` and then creates a new `Reader` instance with the tokens.
/// Then it calls `read_form` with the `Reader` instance.
fn read_str(input: &str) -> Result<MalType, MalError> {
    let mut reader = Reader::new(tokenize(input));
    read_form(&mut reader)
}

/// Takes a single string and returns a vector of all the tokens (strings) in
/// it. The following regular expression will match all mal tokens.
/// ```
/// [\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)
/// ```
///
/// For each of match captured within the parenthesis starting at char 6 of
/// the regular expression a new token will be created.
/// * `[\s,]*`: Matches any number of whitespaces or commas. This is not
///   captured so it will be ignored and not tokenized.
/// * `~@`: Captures the special two-characters `~@` (tokenized).
/// * `[\[\]{}()'\`~^@]`: Captures any special single character, one of
///   `[]{}()'\`~^@` (tokenized).
/// * `"(?:\\.|[^\\"])*"?`: Starts capturing at a double-quote and stops at
///   the next double-quote unless it was preceded by a backslash in which
///   case it includes it until the next double-quote (tokenized). It will
///   also match unbalanced strings (no ending double-quote) which should be
///   reported as an error.
/// * `;.*`: Captures any sequence of characters starting with `;`
///   (tokenized).
/// * `[^\s\[\]{}('"\`,;)]*`: Captures a sequence of zero or more non special
///   characters (e.g. symbols, numbers, "true", "false", and "nil") and is
///   sort of the inverse of the one above that captures special characters
///   (tokenized).
fn tokenize(input: &str) -> impl Iterator<Item=Token> + '_ {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"#)
        .expect("error compiling regex");
    }
    RE
        .captures_iter(input)
        .map(|c| Token(c[1].to_owned()))
}

/// Peeks at the first token in the `Reader` and switches on the first
/// character of that token. If the character is a left paren then
/// `read_list` is called with the `Reader` instance. Otherwise, `read_atom`
/// is called with the `Reader` instance. The return value from `read_from` is
/// a mal data type.
fn read_form<I>(reader: &mut Reader<I>) -> Result<MalType, MalError>
    where I: Iterator<Item=Token>,
{
    let first_token = reader.peek().ok_or(MalError::Eof)?;
    let first_char = first_token.0.chars().next().unwrap();
    match first_char {
        '(' => {
            let _ = reader.next().unwrap();
            Ok(read_list(reader)?)
        },
        _ => Ok(read_atom(reader)),
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
enum MalError {
    Eof,
}

impl fmt::Display for MalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MalError::Eof => write!(f, "EOF"),
        }
    }
}

/// An enum of all the mal data types; scalar (simple/single) data type values.
#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq))]
enum MalType {
    // TODO: should this be isize or i64?
    Integer(isize),
    Symbol(String),
    List(Vec<MalType>),
    //Nil,
    //True,
    //False,
    //String,
    //Keyword,
}

/// Repeatedly calls `read_form` with the `Reader` instance until it
/// encounters a ')' token (if it reaches EOF before reading a ')' then that
/// is an error). It accumulates the results into a vector. Note that
/// `read_list` repeatedly calls `read_form` rather than `read_atom`. This
/// mutually recursive definition between `read_list` and `read_form` is what
/// allows lists to contain lists.
fn read_list<I>(reader: &mut Reader<I>) -> Result<MalType, MalError>
    where I: Iterator<Item=Token>,
{
    let values: Result<Vec<_>, _> = iter::from_fn(|| Some(read_form(reader)))
        .take_while(|value| match value {
            Ok(MalType::Symbol(symbol)) if symbol == &")" => false,
            _ => true,
        })
        .collect();
    Ok(MalType::List(values?))
}

/// Looks at the contents of the token and returns the appropriate scalar
/// (simple/single) data type value.
fn read_atom<I>(reader: &mut Reader<I>) -> MalType
    where I: Iterator<Item=Token>,
{
    let token = reader.next().unwrap();
    if let Ok(integer) = token.0.parse::<isize>() {
        MalType::Integer(integer)
    } else {
        MalType::Symbol(token.0.clone())
    }
}

/// Does the opposite of `read_str`. Takes a mal data structure and returns a
/// string representation of it. But `pr_str` is much simpler and is basically
/// just a switch statement on the type of the input object:
/// * symbol: return the string name of the symbol
/// * number: return the number as a string
/// * list: iterate through each element of the list calling `pr_str` on it,
///   then join the results with a space separator, and surround the final
///   with parens
fn pr_str<'a>(input: &MalType) -> String {
    match &input {
        &MalType::Integer(integer) => integer.to_string(),
        &MalType::Symbol(symbol) => symbol.to_owned(),
        &MalType::List(ref values) => {
            let string_values = values.iter().map(|v| pr_str(v)).collect::<Vec<_>>();
            let joined_values = string_values.join(" ");
            format!("({})", joined_values)
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = " , , , ~@~@";
        let tokens = tokenize(input).collect::<Vec<_>>();
        assert_eq!(tokens, vec![Token("~@".to_owned()), Token("~@".to_owned())]);
    }

    #[test]
    fn test_tokenize_with_trailing_space() {
        let input = "(1 2 ";
        let tokens = tokenize(input).collect::<Vec<_>>();
        assert_eq!(tokens, vec![Token("(".to_owned()), Token("1".to_owned()), Token("2".to_owned())]);
    }

    #[test]
    fn test_read_str() {
        let input = "(1 2";
        assert_eq!(read_str(input), Err(MalError::Eof));
    }

    #[test]
    fn test_read_form() {
        let input = "(1 2";
        let tokens = tokenize(input).collect::<Vec<_>>();
        assert_eq!(tokens, vec![Token("(".to_owned()), Token("1".to_owned()), Token("2".to_owned())]);
    }
}
