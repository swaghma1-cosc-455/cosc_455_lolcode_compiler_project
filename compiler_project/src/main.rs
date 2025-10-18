use std::{env, process}; 
use std::fs::read_to_string; 

/// Trait for a simple lolcompiler front-end. 
/// Errors should cause immediate exit inside the implementation.
/// Given by professor
// pub trait Compiler{
//     // Begin the compilation process (entry point)
//     fn compile(&mut self, source: &str); 

//     // Get the next token from lexical analyzer
//     fn next_token(&mut self) -> String; 

//     // Get the current token being processed 
//     fn current_token(&self) -> String; 

//     // set the current token (typically used internally)
//     fn set_current_token(&mut self, tok: String); 
// }

// pub struct LolcodeCompiler{
//     lexer: SimpleLexicalAnalyzer, 
//     current_tok: String, 
// }

// pub struct SimpleLexicalAnalyzer {
//     input: Vec<char>, 
//     position: usize, 
//     current_build: String,
// }


// impl LolcodeCompiler{
//     pub fn new() -> Self {
//         Self {
//             lexer: SimpleLexicalAnalyzer::new(""), 
//             current_tok: String::new(), 
//         }
//     }

//     fn start (&mut self){
//         let candidate = self.lexer.tokens.pop().unwrap_or_default(); 
//         if self.lexer.lookup(&candidate) {
//             self.current_tok = candidate; 
//         } else if !candidate.is_empty() {
//             eprintln!("Lexical error: '{}' is not a recognized token. ", candidate); 
//             std::process::exit(1); 
//         }
//         else {
//             {
//                 eprintln!("User error: The provided sentence is empty. "); 
//                 std::process::exit(1); 
//             }
//         }
//     }   
// }

// impl SimpleLexicalAnalyzer {
//     pub fn new(source: &str) -> Self {
//         Self {
//             input:source.chars().collect(), 
//             position: 0, 
//             current_build: String::new(),
//             tokens: Vec::new(),     
//         }
//     }

//     pub fn tokenize(&mut self){
//         loop {
//             let c = self.get_char(); 
//             if c == '\0'{
//                 break; 
//         }
//             if c.is_whitespace() {
//                 if 
//             }
//     }
// }
// impl Compiler for LolcodeCompiler {
//     fn compile(&mut self, source: &str) {
//         self.lexer = SimpleLexicalAnalyzer::new(source); 
//         self.lexer.tokenize(); 
//         self.start();
//     }

//     fn next_token(&mut self) -> String {
//         let candidate = self.lexer.tokens.pop().unwrap_or_default(); 
//         if self.lexer.lookup(&candidate) {
//             self.current_tok = candidate.clone(); 
//             candidate
//         } else if self.lexer.tokens.is_empty() {
//             self.current_tok.clear(); 
//             String::new()
//         } else {
//             eprintln!("Lexical error: '{}' is not a recognized token. ", candidate); 
//             std::process::exit(1); 
//         }
//     }

//     fn current_token(&self) -> String {
//         self.current_tok.clone(); 
//     }

//     fn set_current_token(&mut self, tok: String) {
//         self.current_tok = tok; 
//     }
// }
// /// Trait for a simple lexical analyzer.
// /// Implements a character-by-character analysis
// /// from a state machine design.
// pub trait LexicalAnalyzer{
//     /// Return the next character from the input
//     fn get_char(&mut self) -> char; 

//     /// Add a character to the current potential token
//     fn add_char(&mut self, c: char); 

//     /// Lookup a potential token to determine if it is valid
//     /// Returns true if a valid token/lexeme, false otherwise
//     fn lookup(&self, s: &str) -> bool; 
// }

pub struct LolcodeCompiler{
    lexer: LolcodeLexicalAnalyzer,
    current_tok: String, 
}

pub trait Compiler {
    fn compile(&mut self, source: &str);
}
// Lexical analyzer - given by professor
pub trait  LexicalAnalyzer {
    fn get_char(&mut self) -> char; 

    fn add_char(&mut self) -> char; 
} 

pub struct LolcodeLexicalAnalyzer{
    input: Vec<char>, 
    position: usize, 
    current_build: String,
    tokens: Vec<String>, 
}
impl LolcodeLexicalAnalyzer {
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(), 
            position: 0, 
            current_build: String::new(),
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) {
        loop {
            let c = self.get_char(); 

            // end of file contents, terminate the loop
            if c == '\0' {
                break; 
            }

            else if c.is_whitespace() {
                if !self.current_build.is_empty() {
                    self.tokens.push(std::mem::take(&mut self.current_build));
            }
        } else {
            self.add_char(); 
        }
    }

        if !self.current_build.is_empty()
        {
            self.tokens.push(std::mem::take(&mut self.current_build)); 
        }

        self.tokens.reverse();

        println!("{:?}", self.tokens); 
}
}


impl LexicalAnalyzer for LolcodeLexicalAnalyzer{
    fn get_char(&mut self) -> char {
        if self.position >= self.input.len() {
            return '\0'; 
        }
        let c = self.input[self.position]; 
        self.position += 1; 
        c
    }

    fn add_char(&mut self) -> char {
        let c = self.input[self.position - 1]; 
        self.current_build.push(c); 
        c
    }
}
impl LolcodeCompiler {
    pub fn new() -> Self {
        Self {
            lexer: LolcodeLexicalAnalyzer::new(""),
            current_tok: String::new(), 
        }
    }
}
impl Compiler for LolcodeCompiler{
    fn compile(&mut self, source: &str) {
        self.lexer = LolcodeLexicalAnalyzer::new(source); 
        self.lexer.tokenize();
}
}
struct Config {
    file_path: String, 
}
impl  Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments, add a file argument"); 
        }

        let file_path = args[1].clone(); 

        Ok(Config { file_path })
    }
}

fn main() {

    let args: Vec<String> = env::args().collect(); 
    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}"); 
        process::exit(1); 
    }); 

    let lolcode_string: String; 
    match read_to_string(config.file_path) {
        Ok(contents) => lolcode_string = contents,
        Err(e) => {
            println!("Error reading the file: {e}"); 
            process::exit(1); 
        }
    }

    let mut compiler = LolcodeCompiler::new(); 
    compiler.compile(&lolcode_string);
}
