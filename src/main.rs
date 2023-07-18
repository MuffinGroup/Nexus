#[macro_use]
mod tokens;
mod errors;
mod lexer;

use errors::throw_error;
use std::fs::File;
use std::io::Read;

use crate::lexer::lex;

fn main() {
    /*
    let content = read_file("examples/main.nex");
    println!("{}", content);
    */

    let example_code = r#"
        var test1 = 3
        if test == 0 {
            test = 1
        }
    "#;

    let token_stream = lex(example_code);
    for token in token_stream {
        print!("{}", token.to_string());
        print!(", ");
    }
}

#[allow(dead_code)]
fn read_file(path: &str) -> String {
    let mut file = File::open(path).expect("Failed to find file");
    let mut buffer = String::new();
    let file_ending = match path.split('.').last() {
        Some(ending) => ending,
        None => "unknown",
    };

    let error_message = format!("Wrong file format. Current: {}, expected: nex", file_ending);

    if file_ending != "nex" {
        throw_error(&error_message);
    }

    file.read_to_string(&mut buffer)
        .expect("Failed to read file");

    buffer
}
