/// Crates that are used to support the implementation of Lolcode compiler
/***
 * Regex will be used to validate URLS, variable names, variable definitions and text conventions
 * arch::x86_64 has been used to built a binary optimized for x86 architectures (optimized support for chrome)
 * Hashmap will be used to store variable name key pairs within their scopes
 * Fs - file crate used to getting input from file and appending content to a file
 * Env - used to collect command line arguments from the program
 * Process - provide standardized system errors 
 * Path - Handle system file paths for opening files in chrome (copied from the chatgpt response provided by professor)
 * 
 */
use regex::Regex;
use std::arch::x86_64::CpuidResult;
use std::collections::HashMap;
use std::fs::{File, read_to_string};
use std::{env, process, vec, io};
use std::{fs, path::Path, process::Command};


/**
 * Windows specific crates to allow detection of Chrome (using winreg crate)
 * 1. winreg.enums crate - contains enumerations and definitions of all possible windows disposition and registry type values (useful to search Chrome if not already in path)
 * 2. winreg.RegKey crate - provides an interface to directly interact with Registry editor keys in windows 
 */
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/**
 * Initialize a struct (class) for LolCodeCompiler
 * 1. contains a lexer - contains lexical analyzer to parse symbols
 * 2. contains a parse - syntax analyzer to check rules
 * 3. contains a current token variable (will keep track of tokens collected from program string)
 * 4. Scope stack - Will keep track of variables and their scopes
 * 5. Language tokens - Used to store tokens and their line numbers for parsing
 */
pub struct LolcodeCompiler {
    lexer: LolcodeLexicalAnalyzer,
    parser: LolcodeSyntaxAnalyzer,
    current_tok: String,
    scope_stack: Vec<HashMap<String, VariableInfo>>,
    language_tokens: Vec<(String, usize)>,
}

/**
 * 1. #[derive(Clone)] - procedural macro to allow VariableInfo to clone VariableInfo 
 * 2. #[derive(Debug)] - procedural macro to allow VariableInfo to debug (println!) contents of VariableInfo

 */
#[derive(Clone)]
#[derive(Debug)]

/**
 * Variable Info struct 
 * 1. Name - consists of the name of the variable
 * 2. value - consists of the value of the variable
 * 3. line_defined - consists of the line where the variable is defined 
 */
struct VariableInfo {
    name: String,
    value: Option<String>,
    line_defined: usize,
}

/**
 * Compiler trait - required functions of the compiler trait as described in assignment
 * 1. compile - method to break program strings into tokens through character-by-character processing and populate the first token
 * 2. parse - method to start the process of lexically analyzing and parsing program strings token by token starting from first token 
 * 3. next_token - method to get the next input token between lexer and parser
 * 4. current_token - return the current token in the compiler's input bin
 * 5. set_current_token - set a current token to the compiler's input bin
 */
pub trait Compiler {
    fn compile(&mut self, source: &str);
    fn next_token(&mut self) -> String;
    fn parse(&mut self);
    fn current_token(&self) -> String;
    fn set_current_token(&mut self, tok: String);
}

/**
 * Lexical Analyzer trait - required functions of the lexical analyzer trait as described in assignment
 * 1. get_char - function to get next character from program string to form a token
 * 2. add_char - function to add characters to form a token from a program string
 * 3. lookup- utility function that will be used to validate the extracted tokens are valid lexemes in the language
 */
pub trait LexicalAnalyzer {
    fn get_char(&mut self) -> char;
    fn add_char(&mut self) -> char;
    fn lookup(&self, s: &str) -> bool;
}

/**
 * Task 1 - Build a character by character lexical analyzer
 * LolcodeLexicalAnalyzer struct to define LexicalAnalyzer traits 
 * 1. input - vector for all characters extracted from the program string read from lolcode file
 * 2. position - line position for all tokens in the program in the string
 * 3. current_build - placeholder for building tokens through character-by-character reading in compilation
 * 4. tokens - vector holding tuples containing the extracted tokens from lolcode program, this vector will be used for lexical analysis and parsing later in the program
 * 5. line_number - integer value representing a specific line number inside a token
 * 6. head_start - vector to hold starting tag of the document - #hai
 * 7. head_end- vector to hold ending tag of the document - #kthxbye
 * 8. comment_start - vector to hold starting tag of comments - #obtw
 * 9. comment_end - vector to hold ending of comments - #
 * 10. make_start - vector to hold #maek tag for list, paragraf and head
 * 11. oic_end - vector to hold #oic - end tags for list, paragraf and head
 * 12. gimmeh_start - vector to hold #gimmeh - start tags for italics, bold, newline, video, audio, and list item
 * 13. mkay_end - vector to hold #mkay tag - end tags for italics, bold, newline, video, audio, list item and variable use
 * 14. variable_start - vector to include starting portion of variable - #i and #haz used for variable declaration
 * 15. variable_mid - vector to include middle portion of variable declaration - #it and iz 
 * 16. variable_end - vector to include tags for variable_usage between other tags - #lemme and see
 * 17. head_element - vector to include the head tag - used to create head sections of web page
 * 18. title_element - vector to include the title tag - used to create title sections of the web page
 * 19. paragraph_element - vector to include the paragraph tag - used to create paragraf sections of the web page
 * 20. bold_element - vector to include the bold tag - used to create bold text
 * 21. italics_element - vector to include the italics tag - used to create italics text
 * 22. list_element - vector to include the list tag - used to create lists 
 * 23. item_element - vector to include the item tag - used to create list items i
 * 24. newline_element - vector to  include the newline tag, similar to <br> in html
 * 25. soundz_element - vector to include the sound tag in html
 * 26. vidz_element - vector to include the video tag in html
 * 27. var_def - regex expression to enforce variable naming rules
 * 28. var_val - regex expression to enforce allowed variable values
 * 29. text - regex expression to declare acceptable text token
 * 30. address - regex compression to validate URL addresses
 *
 * 
 * 
 */
pub struct LolcodeLexicalAnalyzer {
    input: Vec<char>,
    position: usize,
    current_build: String,
    tokens: Vec<(String, usize)>, // Token and line number
    line_number: usize,
    head_start: Vec<String>,
    head_end: Vec<String>,
    comment_start: Vec<String>,
    comment_end: Vec<String>,
    make_start: Vec<String>,
    oic_end: Vec<String>,
    gimmeh_start: Vec<String>,
    mkay_end: Vec<String>,
    variable_start: Vec<String>,
    variable_mid: Vec<String>,
    variable_end: Vec<String>,
    head_element: Vec<String>,
    title_element: Vec<String>,
    paragraph_element: Vec<String>,
    bold_element: Vec<String>,
    italics_element: Vec<String>,
    list_element: Vec<String>,
    item_element: Vec<String>,
    newline_element: Vec<String>,
    soundz_element: Vec<String>,
    vidz_element: Vec<String>,
    var_def: Regex,
    var_val: Regex,
    text: Regex,
    address: Regex,
}

/***
 * Initialize elements for the impl LolcodeLexicalAnalyzer 
 * 1. input - initialized to hold characters from program string
 * 2. current_build - initialize new string builds
 * 3. tokens - initialize a new vector for created tokens
 * 4. line_number - intiailize line number from one
 * 5. Initialize vectors for all the lolcode compiler tags
 * 6. Regexes defined all four acceptable 
 * i. variable_definition - Any single word (A-Z, a-z, no spaces) - letters only
 * ii. variable_value - Allowed text characters - A-Z, a-xz, 0-9, commas, preiod, period, quotes, colons, question marks, underscores and forward slashes 
 * iii. text - allowed text in our language - A-Z, a-z, 0-9, commas, period, quotes, colons, question marks, underscores, and forward slashes
 * iv. address - allowed text characters without spaces
 */

impl LolcodeLexicalAnalyzer {
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            position: 0,
            current_build: String::new(),
            tokens: Vec::new(),
            line_number: 1,
            head_start: vec!["#hai".into()],
            head_end: vec!["#kthxbye".into()],
            comment_start: vec!["#obtw".into()],
            comment_end: vec!["#tldr".into()],
            make_start: vec!["#maek".into()],
            oic_end: vec!["#oic".into()],
            gimmeh_start: vec!["#gimmeh".into()],
            mkay_end: vec!["#mkay".into()],
            variable_start: vec!["#i".into(), "haz".into()],
            variable_mid: vec!["#it".into(), "iz".into()],
            variable_end: vec!["#lemme".into(), "see".into()],
            head_element: vec!["head".into()],
            title_element: vec!["title".into()],
            paragraph_element: vec!["paragraf".into()],
            bold_element: vec!["bold".into()],
            italics_element: vec!["italics".into()],
            list_element: vec!["list".into()],
            item_element: vec!["item".into()],
            newline_element: vec!["newline".into()],
            soundz_element: vec!["soundz".into()],
            vidz_element: vec!["vidz".into()],
            var_def: Regex::new(r"^[A-Za-z]+$").unwrap(),
            var_val: Regex::new(r"^[A-Za-z0-9,\.\':\?!_\/ ]+$").unwrap(),
            text: Regex::new(r"^[A-Za-z0-9,\.\':\?!_\/ ]+$").unwrap(),
            address: Regex::new(r"^[A-Za-z0-9,\.\':\?!_\/%]+$").unwrap(),
        }
    }

    /***
     * Function to build tokens from characters and append the tuples of tokens and line number to the tokens vector
     */

    pub fn tokenize(&mut self) {
        loop {
            /// get character from program string 
            let c = self.get_char();

            /// end of program string, break the loop
            if c == '\0' {
                break;
            }

            /// If it reaches end of a line
            if c == '\n'
            {
                /// If the current build is not empty, append it as a  token with a line number in the form of tuple to the tokens vector
                if !self.current_build.is_empty() {
                    self.tokens
                        .push((std::mem::take(&mut self.current_build), self.line_number));
                }
                // Go to the next line of program string
                self.line_number += 1;
            } 

            // If whitespace is found, if current_build is not empty, append it as a token with a line number in the form of tuple to the tokens vector
            else if c.is_whitespace() {
                if !self.current_build.is_empty() {
                    self.tokens
                        .push((std::mem::take(&mut self.current_build), self.line_number));
                }
            }
            // Else if there is a non-empty token, then add the character to the token
             else {
                self.add_char();
            }
        }

        // At the end, if the current_build is not empty, add the current_build as a tuple (current_build, line_number) to the tokens vector
        if !self.current_build.is_empty() {
            self.tokens
                .push((std::mem::take(&mut self.current_build), self.line_number));
        }

        // Reverse to get first token when popping from the tokens vector
        self.tokens.reverse();
    }

    // Return the tokens for parsing variables for later html conversion if parsing is syntactically valid
    pub fn return_tokens(&mut self) {
        self.tokens.clone();
    }

    // function to match variable token names based on variable definition rules
    fn is_variable_identifier(&self, s: &str) -> bool {
        self.var_def.is_match(s)
    }
}

impl LexicalAnalyzer for LolcodeLexicalAnalyzer {
    // function to get character from program string 
    
    fn get_char(&mut self) -> char {
        // If position is greater length of program string, return nul characer
        if self.position >= self.input.len() {
            return '\0';
        }
        // get the value based on an index from the input vector
        let c = self.input[self.position];

        // increment the position
        self.position += 1;

        // return the value
        c
    }

    // function to add character to a current token
    fn add_char(&mut self) -> char {

        // get the character from the input vector
        let c = self.input[self.position - 1];

        // append the character to the current build to form a token
        self.current_build.push(c);

        // return the character token
        c
    }

    // function to validate all the lexeme tokens - tags and acceptable text elements - 
    // return false if a token does not match any of these lexeme rules
    fn lookup(&self, s: &str) -> bool {

        //check tags that start with hashtag markup notation
        if s.starts_with("#") {
            return self.head_start.iter().any(|h| h == &s.to_lowercase())
                || self.head_end.iter().any(|h| h == &s.to_lowercase())
                || self.comment_start.iter().any(|h| h == &s.to_lowercase())
                || self.comment_end.iter().any(|h| h == &s.to_lowercase())
                || self.make_start.iter().any(|h| h == &s.to_lowercase())
                || self.oic_end.iter().any(|h| h == &s.to_lowercase())
                || self.gimmeh_start.iter().any(|h| h == &s.to_lowercase())
                || self.mkay_end.iter().any(|h| h == &s.to_lowercase())
                || self.variable_start.iter().any(|h| h == &s.to_lowercase())
                || self.variable_mid.iter().any(|h| h == &s.to_lowercase())
                || self.variable_end.iter().any(|h| h == &s.to_lowercase());
        }

        //check other non element hashtags, and other acceptable text, URL address, variable definition, and variable value formats
        self.head_element.iter().any(|h: &String| h == &s.to_lowercase())
            || self.title_element.iter().any(|h| h == &s.to_lowercase())
            || self
                .paragraph_element
                .iter()
                .any(|h| h == &s.to_lowercase())
            || self.bold_element.iter().any(|h| h == &s.to_lowercase())
            || self.italics_element.iter().any(|h| h == &s.to_lowercase())
            || self.list_element.iter().any(|h| h == &s.to_lowercase())
            || self.item_element.iter().any(|h| h == &s.to_lowercase())
            || self.newline_element.iter().any(|h| h == &s.to_lowercase())
            || self.soundz_element.iter().any(|h| h == &s.to_lowercase())
            || self.vidz_element.iter().any(|h| h == &s.to_lowercase())
            || self.text.is_match(s)
            || self.address.is_match(s)
            || self.var_def.is_match(s)
            || self.var_val.is_match(s)
    }
}



/**
 * Task 2 - Build a recursive descent parser 
 * Trait required to implement as given by project guidelines
 * Methods: 
 * 1. parse_lolcode - parse the structure of lolcode other than #HAI and #KTHXBYE tags
 * 2. parse_head - parse the head portion of the page
 * 3. parse_title - parse the title inside the head portion of the web page
 * 4.  parse_comments - parse the comments section of the web page
 * 5. parse_comment - parse individual comments from the web page
 * 6. parse_body - parse the body of the web page
 * 7. parse_inner_body - parse the inner body of the web page
 * 8. parse_paragraph - parse the paragraph portion of the web page
 * 9. parse_inner_paragraph - parse the inner paragraph portion of the web page
 * 10. parse_list - parse the list portion of the web page
 * 11. parse_list_items - parse the list items of the web page
 * 12. parse_inner_list - parse the inner list items of the web page
 * 13. parse_item - parse the items inside the list of the web page
 * 14. parse_audio - parse the audio tags of the lolcode script
 * 15. parse_video - parse the video tags of the lolcode script
 * 16. parse_newline - parse the newline tags of the lolcode script
 * 17. parse_bold - parse the bold tags of the lolcode script
 * 18. parse_italics - parse the italics tags of the lolcode script
 * 19. parse_text - parse the text tags of the lolcode script
 * 20. parse_inner_text - parse the inner text of the lolcode script
 * 21. parse_variable_define - parse the variable definition of the lolcode script
 * 22. parse_variable_use - parse the variable usage of the lolcode script
 */
pub trait SyntaxAnalyzer {
    fn parse_lolcode(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_head(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_title(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_comments(&mut self, compiler: &mut LolcodeCompiler); 
    fn parse_comment(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_body(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_inner_body(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_paragraph(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_inner_paragraph(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_list(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_list_items(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_inner_list(&mut self, compiler: &mut LolcodeCompiler);

    fn parse_item(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_audio(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_video(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_newline(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_bold(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_italics(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_text(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_inner_text(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_variable_define(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_variable_use(&mut self, compiler: &mut LolcodeCompiler);
}

// Struct definition of parser, containing current_line to represent the line of a given token
pub struct LolcodeSyntaxAnalyzer {
    current_line: usize,
}

// Implementation for lolcode syntax analyzer methods, contains utility method 
impl LolcodeSyntaxAnalyzer {
    pub fn new() -> Self {
        Self { current_line: 1 }
    }

    /// Helper methods to check token types using compiler's lexer elements which contain the allowed lexemes
    

    /// check if the token at the end of element is #kthxbye
    fn is_document_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.head_end.iter().any(|head| head == &s.to_lowercase())
    }

    /// check if the token entered is #maek token
    fn is_make_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .make_start
            .iter()
            .any(|make| make == &s.to_lowercase())
    }

    /// check if the token entered is an #oic token - used for ending heading, paragraf and list
    fn is_oic_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.oic_end.iter().any(|oic| oic == &s.to_lowercase())
    }

    /// check if the token entered is a #gimmeh token 
    fn is_gimmeh_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .gimmeh_start
            .iter()
            .any(|gimmeh| gimmeh == &s.to_lowercase())
    }

    /// check if the token entered is a mkay token used for ending some lolcode tags
    fn is_mkay_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.mkay_end.iter().any(|mkay| mkay == &s.to_lowercase())
    }

    /// check if the token entered represents start of a comment - #obtw
    fn is_comment_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .comment_start
            .iter()
            .any(|comment| comment == &s.to_lowercase())
    }

    /// check if the token entered represents end of a comment - #tldr
    fn is_comment_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .comment_end
            .iter()
            .any(|comment| comment == &s.to_lowercase())
    }

    /// check if the token entered represents start of a variable definition - #I HAZ
    fn is_variable_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.variable_start.iter().any(|v| v == &s.to_lowercase())
    }

    /// check if the token entered represents mid part of variable definition - #it iz
    fn is_variable_mid(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.variable_mid.iter().any(|v| v == &s.to_lowercase())
    }

    /// check if the token entered represents variable usage definiton - #lemme see
    fn is_variable_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.variable_end.iter().any(|v| v == &s.to_lowercase())
    }

    /// check if the token entered represents head element - head
    fn is_head_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .head_element
            .iter()
            .any(|head| head == &s.to_lowercase())
    }

    /// check if the token entered represents title element - title
    fn is_title_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .title_element
            .iter()
            .any(|title| title == &s.to_lowercase())
    }

    /// check if the token entered represents paragraf element - paragraf
    fn is_paragraph_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .paragraph_element
            .iter()
            .any(|para| para == &s.to_lowercase())
    }

    /// check if the token entered represents bold element - bold
    fn is_bold_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .bold_element
            .iter()
            .any(|bold| bold == &s.to_lowercase())
    }

    /// check if the token entered represents italics element - italicz
    fn is_italics_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .italics_element
            .iter()
            .any(|italic| italic == &s.to_lowercase())
    }

    /// check if the token entered represents list element - list
    fn is_list_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .list_element
            .iter()
            .any(|list| list == &s.to_lowercase())
    }

    /// check if the token entered represent item element - item
    fn is_item_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .item_element
            .iter()
            .any(|item| item == &s.to_lowercase())
    }

    /// check if the token entered represents newline element - newline
    fn is_newline_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .newline_element
            .iter()
            .any(|nl| nl == &s.to_lowercase())
    }

    /// check if the token entered represents soundz element - soundz
    fn is_soundz_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .soundz_element
            .iter()
            .any(|sound| sound == &s.to_lowercase())
    }

    /// check if the token entered represents vidz element - vidz
    fn is_vidz_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .vidz_element
            .iter()
            .any(|vid| vid == &s.to_lowercase())
    }

    /// check if the token entered matches accepted tokens allowed in text of the language 
    fn is_text(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.text.is_match(s)
    }

    /// check if the token entered matches web URLS found in audio or video hyperlinks 
    fn is_address(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.address.is_match(s)
    }

    /// check if the token entered matches variable identifier rules
    fn is_variable_identifier(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.is_variable_identifier(s)
    }

  
}

impl SyntaxAnalyzer for LolcodeSyntaxAnalyzer {
    // Parse the #HAI tag at the start of document, report a syntax error if it is missing
    fn parse_lolcode(&mut self, compiler: &mut LolcodeCompiler) {
        
        //Parse comments if any comments are found
            self.parse_comments(compiler);


        // Allow variable declarations before head
        while self.is_variable_start(&compiler.current_tok, &compiler.lexer) {
                 self.parse_variable_define(compiler);
        }

        // Parse head elements if any head elements are found
        self.parse_head(compiler);

        // Parse body elements if any body elements are found
        self.parse_body(compiler);
        

}

    // Parse comments by going through each individual comment as described in BNF grammar
    fn parse_comments(&mut self, compiler: &mut LolcodeCompiler) {
        while self.is_comment_start(&compiler.current_tok, &compiler.lexer)
        {
            self.parse_comment(compiler);
        }

    }

    // Parse head element by going through components of the head element - requires a #maek tag, head element, title element, and oic
    fn parse_head(&mut self, compiler: &mut LolcodeCompiler) {

        // Expect #MAEK, if #MAEK not found report a syntax error
        if !self.is_make_start(&compiler.current_tok, &compiler.lexer){
            eprintln!(
                "Syntax error at line {}: Expected '#maek', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //get the next token from the compiler
        compiler.current_tok = compiler.next_token();

        // Expect HEAD, if HEAD not found report a syntax error
        if !self.is_head_element(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected 'head', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //get the next token from the compiler
        compiler.current_tok = compiler.next_token();

        // Parse title - described later in the code
        self.parse_title(compiler);

        // Expect #OIC, if #oic not found report a syntax error
        if !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#oic', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //get the next token from the compiler
        compiler.current_tok = compiler.next_token();


        //If head not found, skip this function
    }

    //Parse title based on its definition given in BNF, needs #gimmeh, title tag, title text and mkay tag
    fn parse_title(&mut self, compiler: &mut LolcodeCompiler) {

        // Expect #GIMMEH, if #gimmeh is not found - report an error
        if !self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#gimmeh', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //get next token from the compiler
        compiler.current_tok = compiler.next_token();

        // Expect TITLE, if title is not found - report an error
        if !self.is_title_element(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected 'title', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //get next token from the compiler
        compiler.current_tok = compiler.next_token();

        // Consume text until #MKAY tag is found using parse text method, report an error if token is found empty
        while !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in title.",
                    self.current_line
                );
                std::process::exit(1);
            }
            
            //consumre text tokens
            self.parse_text(compiler);
        }

        // Consume #MKAY at the end
        compiler.current_tok = compiler.next_token();
    }

    // parse individual comments - look for #obtw text #tldr
    fn parse_comment(&mut self, compiler: &mut LolcodeCompiler) {

    
    // Expect #obtw if not found - report an error
    if !self.is_comment_start(&compiler.current_tok, &compiler.lexer) {
        eprintln!(
            "Syntax error at line {}: Expected comment start '#obtw', found '{}'.",
            self.current_line, compiler.current_tok
        );
        std::process::exit(1);
    }
    
    // get the next token from the compiler
    compiler.current_tok = compiler.next_token();
    
    // get the text tokens from the compiler
    self.parse_text(compiler);
    
    // Expect #tldr at the end of comment, if not found - report an error
    if !self.is_comment_end(&compiler.current_tok, &compiler.lexer) {
        eprintln!(
            "Syntax error at line {}: Expected comment end '#tldr', found '{}'.",
            self.current_line, compiler.current_tok
        );
        std::process::exit(1);
    }
    
    // get the next token from the compiler
    compiler.current_tok = compiler.next_token();
 
}

// parse the body of the lolcode script till the #kthxbye tag as given in BNF
    fn parse_body(&mut self, compiler: &mut LolcodeCompiler) {
        // Parse body elements until we hit #KTHXBYE
        if !self.is_document_end(&compiler.current_tok, &compiler.lexer) 
        {
            //parse the inner body
            self.parse_inner_body(compiler); 

            //recursive call to the function - if it is empty, it is acceptable
            self.parse_body(compiler);

        }
            
    }

// parse the inner body defined in the parse_body, contains variable definition, paragraf, list, bold, italicz, sound, video, newline elements, variable usage, comments, and text
   fn parse_inner_body(&mut self, compiler: &mut LolcodeCompiler) {
 
    // Don't call next_token here - we already have the current token from parse_body
    
    // If a variable is defined, parse it here
    if self.is_variable_start(&compiler.current_tok, &compiler.lexer) {
        return;
    }        
    // else if the token found is  #maek tag, it can be either a paragraf or a list
    else if self.is_make_start(&compiler.current_tok, &compiler.lexer) {
        // Consume #MAEK and get the block type
        compiler.current_tok = compiler.next_token();
        
       
        // If it is a paragraf tag, parse it as a paragraf
        if self.is_paragraph_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_paragraph(compiler);
        }

        // If it is a paragraf tag, parse it as a list
        else if self.is_list_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_list(compiler);
        }

        // Report an error if #maek is found and there is neither paragraf nor list
        else {
            eprintln!(
                "Syntax error at line {}: Expected 'paragraf' or 'list', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        return; 
    }
    // If the next token found is #gimmeh, 
    else if self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {

        //get the next token to determine which tag it its
        compiler.current_tok = compiler.next_token(); 

        // if it is a bold element, parse it using the bold element, here #gimmeh and bold are checked before sending it to parse_bold
        if self.is_bold_element(&compiler.current_tok, &compiler.lexer) {
            compiler.current_tok = compiler.next_token(); 
            self.parse_bold(compiler);
            return;
        } 

        // if it is an italics element, parse it using the italics element, here #gimmeh and italics are checked before sending it to parse_italics
        else if self.is_italics_element(&compiler.current_tok, &compiler.lexer) {
            compiler.current_tok = compiler.next_token(); 
            self.parse_italics(compiler);
            return;
        }

        //if it is a soundz element, parse it using the sound element, and return back 
        else if self.is_soundz_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_audio(compiler);
            return;
        }
    
        //if it is a vidz element, parse it using the sound element, and return back 
        else if self.is_vidz_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_video(compiler);
            return;
        }

         //if it is a newline element, parse it using the newline element, and return back 
        else if self.is_newline_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_newline(compiler);
            return;
        }

        //return an error if #gimmeh is found and no bold, italics, soundz, vidz, or newline is found
        else {
            eprintln!(
                "Syntax error at line {}: Expected 'bold', 'italics', 'soundz', 'vidz' or 'newline', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
    }

    //parse variable usage part if it is found
    else if self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
        self.parse_variable_use(compiler);
    }

    //parse a comment if a comment is found
    else if self.is_comment_start(&compiler.current_tok, &compiler.lexer) {
        self.parse_comment(compiler);

    }

    //if token does not match anything, is not empty, and is not a tag,it must be an acceptable text token, parse it as a text
    else if !compiler.current_tok.is_empty() {
        self.parse_text(compiler);
    }
}

// parse the paragraf method and contents inside paragraf
  fn parse_paragraph(&mut self, compiler: &mut LolcodeCompiler) {
  
    // Already consumed #MAEK, current_tok is PARAGRAF

    //push the variable scope in scope stack on entering a new paragraf tag 
    compiler.push_scope();

    // Verify we're on PARAGRAF, else report an error to paragraf
    if !self.is_paragraph_element(&compiler.current_tok, &compiler.lexer) {
        eprintln!(
            "Syntax error at line {}: Expected 'paragraf', found '{}'.",
            self.current_line, compiler.current_tok
        );
        std::process::exit(1);
    }
    
    // Consume PARAGRAF and move to the paragraph content
    compiler.current_tok = compiler.next_token();
  

    // Parse paragraph contents till the #oic end tag is found

    while !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
        // Report an error if tokens found are empty
        if compiler.current_tok.is_empty() {
            eprintln!(
                "Syntax error at line {}: Unexpected end of input in paragraph.",
                self.current_line
            );
            std::process::exit(1);
        }

        //parse the variable definition there is one found subsequently as defined in BNF
        if self.is_variable_start(&compiler.current_tok, &compiler.lexer) {
            self.parse_variable_define(compiler);
            // parse_variable_define already advances token, continue loop
        }
        else {

            // Parse the content and advance
            self.parse_inner_paragraph(compiler);
        }
    }

    // Consume #OIC else report an error if it is not found
    if !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
        eprintln!(
            "Syntax error at line {}: Expected '#oic', found '{}'.",
            self.current_line, compiler.current_tok
        );
        std::process::exit(1);
    }

    //get the next token from the compiler
    compiler.current_tok = compiler.next_token();
    
    //Remove the scope from the scope stack after going out of paragraf tag
    compiler.pop_scope();
}

// parse inner_paragraf and its contents which include inner_text
   fn parse_inner_paragraph(&mut self, compiler: &mut LolcodeCompiler) {
  
    
    // Parse one element of paragraph content 
    self.parse_inner_text(compiler);
    
    // Advance to next token, till the end
    if !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
        compiler.current_tok = compiler.next_token();
    }
}

//Parse the list found, if any, inside the paragraf
    fn parse_list(&mut self, compiler: &mut LolcodeCompiler) {
        
        //Expect #maek, if not found -> report an error
        if !self.is_make_start(&compiler.current_tok, &compiler.lexer)
        {
              eprintln!(
                "Syntax error at line {}: Expected '#maek', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //get the next token from the user
        compiler.current_tok = compiler.next_token();

        // if list element not found, report an error 
         if !self.is_list_element(&compiler.current_tok, &compiler.lexer)
        {
              eprintln!(
                "Syntax error at line {}: Expected 'list', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        // get the next token from the compiler
        compiler.current_tok = compiler.next_token();

        //parse the list items inside the list
        self.parse_list_items(compiler);

        // Expect #OIC at the end of list, else report an error
        if !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!(
                    "Syntax error at line {}: Expected 'oic', found '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }
        }

        // Consume #OIC, get the next token from the compiler
        compiler.current_tok = compiler.next_token();
    }

    //function to parse list items
    fn parse_list_items(&mut self, compiler: &mut LolcodeCompiler)
    {
        // if compiler token is non-empty
        if !compiler.current_tok.is_empty()
        {
            // parse a single list item
            self.parse_item(compiler);

            //recursive call to parse_list_items if no token found, return control back to calling function
            self.parse_list_items(compiler);

        }
    }

    //function to parse inner text which include - variable usage, bold, italicz, newline, soundz, vidz, list and text
    fn parse_inner_text(&mut self, compiler: &mut LolcodeCompiler) {

    // If variable usage is found, parse it accordinglya and get the next token
    if self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
        self.parse_variable_use(compiler);
    }

    //if #gimmeh is found, check to see if it is bold, italicz, newline, sounds, vidz
    else if self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {

        //get the next token from gimmeh to determine what it is
        compiler.current_tok = compiler.next_token();
        
        //If it is bold, call the bold function, #gimmeh and bold already comsumed
        if self.is_bold_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_bold(compiler);
        } 

        //If it is italicz, call the italics function, #gimmeh and italics already comsumed
        else if self.is_italics_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_italics(compiler);
        }

        //If it is newline, call the newline function, #gimmeh and newline already comsumed
        else if self.is_newline_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_newline(compiler);
        }

        //If it is soundz, call the soundz function, #gimmeh and soundz already comsumed
        else if self.is_soundz_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_audio(compiler);
        } 

        //If it is vidz, call the vidz function, #gimmeh and vidz already comsumed
        else if self.is_vidz_element(&compiler.current_tok, &compiler.lexer) {
            self.parse_video(compiler);
        }

        //report an error if anything else is found after #gimmeh except the above tags
        else {
            eprintln!(
                "Syntax error at line {}: Expected 'bold', 'italics', 'newline', 'soundz', or 'vidz', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
    }

    //if #maek tag is found, it will be a list
    else if self.is_make_start(&compiler.current_tok, &compiler.lexer) {
        
        //get the next token from the compiler
        compiler.current_tok = compiler.next_token();

        //parse the list appropriately 
        self.parse_list(compiler); 
    }

    //If the token is non-empty and is not a tag (does not start with "#"), consume it as a text element
    else if !compiler.current_tok.starts_with("#") {
        self.parse_text(compiler);
    }
}

// parse the acceptable tokens in the language except tags with #, and some keywords
    //report an error if acceptable tokens are not found
    fn parse_text(&mut self, compiler: &mut LolcodeCompiler) {
        while !compiler.current_tok.starts_with("#") && !self.is_mkay_end(&compiler.current_tok, &compiler.lexer)
        {  
            if compiler.current_tok.is_empty() {
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in bold.",
                    self.current_line
                );
                std::process::exit(1);
            }

            //get the next token from the compiler
            compiler.current_tok = compiler.next_token();
        }

        return; 
    }

    //function to parse list items inside a list, will contain a #gimmeh item variable definition text followed by mkay
    fn parse_item(&mut self, compiler: &mut LolcodeCompiler) {

        // consume #gimmeh, if not found report an error
          if !self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer)
        {
            eprintln!(
                    "Syntax error at line {}: expected #gimmeh found {}",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
        }

        //get the next token from the compiler
        compiler.current_tok = compiler.next_token();

        //consume item, if not found report an error
  if !self.is_item_element(&compiler.current_tok, &compiler.lexer)
        {
            eprintln!(
                    "Syntax error at line {}: expected #gimmeh found {}",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
        }

        //get the next token from the user
        compiler.current_tok = compiler.next_token();

        //function to parse the inner list
        self.parse_inner_list(compiler); 
        

        //consume mkay, if not found report an error
        if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer)
        {
            eprintln!(
                    "Syntax error at line {}: expected #mkay found {}",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
        }

    }


    //functino to parse an inner list, contains bold, italicz, variable usage and text
    fn parse_inner_list(&mut self, compiler: &mut LolcodeCompiler)
    {
        // If the compiler token is not empty
        if !compiler.current_tok.is_empty()
        {
            //if the token given is #gimmeh, look whether it is bold or italics
            if self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer)
            {
                //get the next token to see if it bold or italics
                compiler.current_tok = compiler.next_token();

                // if it is bold, parse the bold element appropriately, #gimmeh and bold already consumed
                if self.is_bold_element(&compiler.current_tok, &compiler.lexer)
                {
                    self.parse_bold(compiler);
                } 
                
                // if it is italicz, parse the italicz element appropriately, #gimmeh and italicz already consumed
                else if self.is_italics_element(&compiler.current_tok, &compiler.lexer)
                {
                    self.parse_italics(compiler);
                }
            }

            // if there is text, parse the text element accordingly
            self.parse_text(compiler);

            //if there is variable usage defined, parse it appropriately
            self.parse_variable_use(compiler);
        }
    }


    // parse the audio element, consists of #gimmeh, audio, link address and mkay tags
    fn parse_audio(&mut self, compiler: &mut LolcodeCompiler) {
        
        // Expect #gimmeh - if not found report an error
        if !self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer)
        {
            eprintln!(
                "Syntax error at line {}: Expected '#gimmeh', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        // get the next token from the compiler
        compiler.current_tok = compiler.next_token(); 


        // expect soundz element - if not found report an error
        if !self.is_soundz_element(&compiler.current_tok, &compiler.lexer)
        {
            eprintln!(
                "Syntax error at line {}: Expected 'soundz', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        // get the next token from the compiler
        compiler.current_tok = compiler.next_token(); 

        // Expect address
        if !self.is_address(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected address for audio, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        // get the next token from the user
        compiler.current_tok = compiler.next_token();

        // Expect #MKAY, if not found report an error
        if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#mkay' after audio address, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //get the next token from the user
        compiler.current_tok = compiler.next_token();
    }

    // parse the vidz element, consists of #gimmeh vidz URL address and mkay at the end
    fn parse_video(&mut self, compiler: &mut LolcodeCompiler) {

        // expect vidz, if not found report an error
        if !self.is_vidz_element(&compiler.current_tok, &compiler.lexer)
        {
            eprintln!(
                "Syntax error at line {}: Expected 'vidz', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        // get the next token from the compiler
        compiler.current_tok = compiler.next_token(); 

        // Expect address, report an error if not found
        if !self.is_address(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected address for audio, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        // get the next token from the compiler
        compiler.current_tok = compiler.next_token();

        // Expect #MKAY, if not found report an error
        if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#mkay' after audio address, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //get the next token from the user
        compiler.current_tok = compiler.next_token();
    }

    //parse a newline tag,has a form #gimmeh newline,  #gimmeh consumed already from parent functions
    fn parse_newline(&mut self, compiler: &mut LolcodeCompiler) {
      
        //Expect #gimmeh, if not found report an error
        if !self.is_newline_element(&compiler.current_tok, &compiler.lexer)
        {
            eprintln!(
                "Syntax error at line {}: Expected 'newline', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
    }

    //parse a bold function, has a form #gimmeh bold text variable_def #mkay, #gimmeh consumed from parent functions
    fn parse_bold(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH from previous functions


        //Expect bold, if not found report an error
        if !self.is_bold_element(&compiler.current_tok, &compiler.lexer)
        {
            eprintln!(
                    "Syntax error at line {}: expected bold found {}",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
        }

        //get the next token from the compiler
        compiler.current_tok = compiler.next_token();

       
        //if variable usage is found, parse it accordingly
            if self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
                self.parse_variable_use(compiler);
            } else {

                //If non-tag text is found, parse it as text
                self.parse_text(compiler);
            }
        

        // Consume #MKAY to signal end of bold element
        compiler.current_tok = compiler.next_token();
    }

    //parse a italicz function, has a form #gimmeh italicz text variable_def #mkay, #gimmeh consumed from parent functions
    fn parse_italics(&mut self, compiler: &mut LolcodeCompiler) {
        //Already consumed #GIMMEH

        //expect #italicz, if not found report an error
        if !self.is_italics_element(&compiler.current_tok, &compiler.lexer)
        {
            eprintln!(
                    "Syntax error at line {}: expected italics found {}",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
        }

        //get the next token from the compiler
        compiler.current_tok = compiler.next_token();

        // Parse variable definition if found one
            if self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
                self.parse_variable_use(compiler);
            } 
            //Parse text if no tags are found
            else {
                self.parse_text(compiler);
            }
        

        // Consume #MKAY to signal end of italicz element
        compiler.current_tok = compiler.next_token();
    }

    //Function to parse variable definition, has a form #i haz variable_name #it iz variable_definition
    fn parse_variable_define(&mut self, compiler: &mut LolcodeCompiler) {
        
        //get next token from the compiler and convert it to lowercase
        let var_keyword = compiler.current_tok.to_lowercase();
      

        // If we saw #I, expect HAZ
        if var_keyword == "#i" {

            //If there is not haz, report a syntax error
            if compiler.current_tok.to_lowercase() != "haz" {
                eprintln!(
                    "Syntax error at line {}: Expected 'haz' after '#i', found '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }

            //get the next token from the compiler
            compiler.current_tok = compiler.next_token();

        }

        // Expect variable identifier to validate variable_name follows naming conventions, if it is empty or does not follow naming rules, report a syntax error
        if !self.is_variable_identifier(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected variable identifier, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //Consume the variable name for storing it in scope stack
        let var_name = compiler.current_tok.clone();

        //get the next token from the compiler
        compiler.current_tok = compiler.next_token();



        // Check for #IT IZ (value assignment)
        let var_value = if self.is_variable_mid(&compiler.current_tok, &compiler.lexer) {

            // convert the keyword into lowercase
            let mid_keyword = compiler.current_tok.to_lowercase();

            //get the next token from user
            compiler.current_tok = compiler.next_token();

            // If we saw #IT, expect IZ
            if mid_keyword == "#it" {

                //if iz is not found, report an error
                if compiler.current_tok.to_lowercase() != "iz" {
                    eprintln!(
                        "Syntax error at line {}: Expected 'iz' after '#it', found '{}'.",
                        self.current_line, compiler.current_tok
                    );
                    std::process::exit(1);
                }

                //get next token from the user
                compiler.current_tok = compiler.next_token();
            }

            // Expect value in the form of text or acceptable text items without spaces, report an error if no such value is found
            if !self.is_text(&compiler.current_tok, &compiler.lexer)
                && !self.is_address(&compiler.current_tok, &compiler.lexer)
            {
                eprintln!(
                    "Syntax error at line {}: Expected value after 'iz', found '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }

            //Consume the value of the variable
            let value = compiler.current_tok.clone();

            //Get the next token from the compiler
            compiler.current_tok = compiler.next_token();

            //get the #mkay token, if not found, report an error
            if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
                eprintln!(
                    "Syntax error at line {}: Expected '#mkay' after variable value, found '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }
            //Add statement for mkay

            compiler.current_tok = compiler.next_token();

            //Include an option to store value of variable
            Some(value)
        } else {
            //If no value found, assign none
            None
        };

        //function to handle semantic analysis - described later in the code
        compiler.declare_variable(var_name, var_value, self.current_line);
       
    }

        //Function to parse variable usage, has a form #lemme see variable_name mkay
    fn parse_variable_use(&mut self, compiler: &mut LolcodeCompiler) {

        // Expect #LEMME , if not found report a syntax error #lemme not found
        if !self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#lemme' or 'see', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        // Get the variable name after #lemme
        let var_keyword = compiler.current_tok.to_lowercase();
        compiler.current_tok = compiler.next_token();

        // If we saw #LEMME, expect SEE
        if var_keyword == "#lemme" {

            //If see not found, report an error
            if compiler.current_tok.to_lowercase() != "see" {
                eprintln!(
                    "Syntax error at line {}: Expected 'see' after '#lemme', found '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }

            //get the next token from the compiler
            compiler.current_tok = compiler.next_token();
        }

        // Expect variable identifier, if missing report an error
        if !self.is_variable_identifier(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected variable identifier, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        //Get the variable name as the next token
        let var_name = compiler.current_tok.clone();

        // Check if variable is defined using lookup_variable, if already defined report an error, or if not defined report an error
        if compiler.lookup_variable(&var_name).is_none() {
            eprintln!(
                "Semantic error at line {}: Variable '{}' is used before being defined.",
                self.current_line, var_name
            );
            eprintln!(
                "  --> Variable '{}' has not been declared in the current scope.",
                var_name
            );
            eprintln!(
                "  --> Use '#I HAZ {}' or 'HAZ {}' to declare the variable before using it.",
                var_name, var_name
            );
            std::process::exit(1);
        }

        //Variable defined successfully, get the next token
        compiler.current_tok = compiler.next_token();

        //If next token not mkay, report an error 
        if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#mkay' after variable usage, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        // get the next token from the compiler
        compiler.current_tok = compiler.next_token();
    }
}

//Implementation for lolcode compiler
impl LolcodeCompiler {
    pub fn new() -> Self {
        Self {
            lexer: LolcodeLexicalAnalyzer::new(""),
            parser: LolcodeSyntaxAnalyzer::new(),
            current_tok: String::new(),
            scope_stack: vec![HashMap::new()],
            language_tokens: vec![],
        }
    }

    // Get the first token and validate if it is empty
    fn start(&mut self) {
        // Get the first token
        self.current_tok = self.next_token();

        // Report an error if it is empty
        if self.current_tok.is_empty() {
            eprintln!("User error: The provided sentence is empty.");
            std::process::exit(1);
        }
    }

    // Parse the lolcode document
    fn lolcode(&mut self) {
        // Document should start with #HAI, if not report an error
        if !self
            .lexer
            .head_start
            .iter()
            .any(|h| h == &self.current_tok.to_lowercase())
        {
            eprintln!(
                "Syntax error at line {}: Expected document start '#hai', found '{}'.",
                self.parser.current_line, self.current_tok
            );
            std::process::exit(1);
        }

        // get the next token from the compiler
        self.current_tok = self.next_token();

        // Parse the document structure
        //Initialize the parser
        let mut parser = std::mem::replace(&mut self.parser, LolcodeSyntaxAnalyzer::new());

        // Parse the lolcode document with parser
        parser.parse_lolcode(self);

        //Assign the parser to the object
        self.parser = parser;

        // Document should end with #KTHXBYE, report an error if #kthxbye not found at the end
        if !self
            .lexer
            .head_end
            .iter()
            .any(|h| h == &self.current_tok.to_lowercase())
        {
            eprintln!(
                "Syntax error at line {}: Expected document end '#kthxbye', found '{}'.",
                self.parser.current_line, self.current_tok
            );
            std::process::exit(1);
        }
    }

    /*****
     * Task 3 - Semantic Analyzer part - Static variable scope resolution methods
     * 1. Push scope - Push the new variable scope into the stack
     * 2. Pop scope - Remove the new variable scope from the stack
     * 3. Declare variable - semantic analyzer validating variable re-declaration and assignment to scope
     * 4. lookup - function supporting retrieval of the values of variable defined in the scopre
     */

    // Push the new scope to the stack
    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());

    }

    // Pop the scope from the stack if there are more than one stack, one scope for global variables 
    fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    // Declare a variable in the current scope with semantic analysis to validate for re-declaration and insert it into scope stack
    fn declare_variable(&mut self, name: String, value: Option<String>, line: usize) {
       
       //Check if there is any variable with the same name in the current scope, if so report an error
        if let Some(current_scope) = self.scope_stack.last_mut() {
            if current_scope.contains_key(&name) {
                let existing = &current_scope[&name];
                eprintln!(
                    "Semantic error at line {}: Variable '{}' is already defined at line {} in the current scope.",
                    line, name, existing.line_defined
                );
                std::process::exit(1);
            }

            //Validation complete, insert the variable into the current scope
            current_scope.insert(
                name.clone(),
                VariableInfo {
                    name,
                    value,
                    line_defined: line,
                },
            );

        }
            

    }

    //Function to retrieve values of the variables, retrieves the value from the innermost scope for a variable
    fn lookup_variable(&self, name: &str) -> Option<&VariableInfo> {
        // Search from innermost to outermost scope, switch to outerscope if value not found in local scope
        for scope in self.scope_stack.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                return Some(var_info);
            }
        }
        //return None if value not found
        None
    }

    /**
     * Task 4 - HTML Conversion - convert the syntactically and semantically valid lolcode into HTML
     */
    fn to_html(&mut self) -> String{

        // Define a scope stack to support resolution of variables
        let mut scope_stack: Vec<VariableInfo> = Vec::new(); 

        // HTML code conversion, get a copy of tokens from the compiler for HTML conversion, already validated
        let tokens = &self.language_tokens;

        // collect the token strings
        let mut token_strings: Vec<String> =
            tokens.iter().map(|(token, _line)| token.clone()).collect();


        //Initialize an empty html string
        let mut html_string: String = " ".to_string();

        //Get the first token
        while let Some(token) = token_strings.pop() {
            // If the first token is #hai, append DOCTYPE and starting html tags
            if token.to_lowercase() == "#hai" {
                html_string.push_str("<!DOCTYPE html> \n<html>");
                continue;

                
            }

            //If there is variable initialization, push the appropriate value into stack
            //If the next is #I
            if token.to_lowercase() == "#i" {
                //Pop next token
                if let Some(token) = token_strings.pop()  {
                    //Expect haz
                    if token.to_lowercase() == "haz"
                    {
                //Pop next token
                            if let Some(next_token) = token_strings.pop()
                        {
                            //Expect variable name
                            let var_name = next_token; 

                            //Pop next token
                            if let Some(next_token) = token_strings.pop()
                            {
                                //Expect #it
                                if next_token.to_lowercase() == "#it"
                                {
                                    //Pop next token
                                    if let Some(next_token_iz) = token_strings.pop()
                                    {
                                        //Expect iz
                                        if next_token_iz.to_lowercase() == "iz"
                                        {
                                            //Pop next token
                                            if let Some(value_token) = token_strings.pop()
                                            {
                                                //Variable value
                                                let var_value = value_token; 

                                                if let Some(mkay_value) = token_strings.pop()
                                                {
                                                    if mkay_value.to_lowercase() == "#mkay"
                                                    {
                                                        //Declare stack, append name and value of variable to VariableInfo structure
                                                        let variable_info = VariableInfo {
                                                            name: var_name,
                                                            value: Some(var_value),
                                                            line_defined: 0,
                                                        };

                                                        //Push the variable to the scope stack
                                                        scope_stack.push(variable_info);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // If the token is #kthxbye, append ending html tag
            if token.to_lowercase() == "#kthxbye" {
                html_string.push_str("\n</html>");
                break;
            }

            //If the token is #obtw, add html comments
            if token.to_lowercase() == "#obtw" {
                html_string.push_str("\n<!--");
                //Consume text tokens
                while let Some(comment_token) = token_strings.pop() {
                    // End when #tldr is found 
                    if comment_token.to_lowercase() == "#tldr" {
                        html_string.push_str(" -->\n");
                        break;
                    }

                    //push comments to html string
                    html_string.push_str(" ");
                    html_string.push_str(&comment_token);
                }
            }

            // if the token is maek
            if token.to_lowercase() == "#maek" {
                //Pop next token
                if let Some(next_token) = token_strings.pop() {
                    //if next token is head, append head
                    if next_token.to_lowercase() == "head" {
                        html_string.push_str("\n<head>");

                        //append end tag of head
                        while let Some(head_token) = token_strings.pop() {
                            if head_token.to_lowercase() == "#oic" {
                                html_string.push_str("</head>\n");
                                break;
                            }

                            //if next token is #gimmeh, append title tag
                            if head_token.to_lowercase() == "#gimmeh" {
                                if let Some(title_token) = token_strings.pop() {
                                    if title_token.to_lowercase() == "title" {
                                        html_string.push_str("\n<title>");
                                        
                                        //consume text tokens for title
                                        while let Some(text_token) = token_strings.pop() {

                                            //append end tag of title when mkay is found
                                            if text_token.to_lowercase() == "#mkay" {
                                                html_string.push_str("</title>\n");
                                                break;
                                            }

                                            //Push title to html string
                                            html_string.push_str(" ");
                                            html_string.push_str(&text_token);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    //If the next element found is paragraf, append starting paragraph tag
                    if next_token.to_lowercase() == "paragraf" {
                        html_string.push_str("\n<p>");

                        //Consume text tokens in paragraph
                        while let Some(para_token) = token_strings.pop() {

                            //if end of paragraph is hit, append ending p tag
                            if para_token.to_lowercase() == "#oic" {
                                html_string.push_str("</p>\n");

                                //If there is any local scope, pop out of the scope stack
                            if scope_stack.len() > 1
                            {
                                scope_stack.pop();
                            }
                                break;
                            }

                            //If there is variable declaration, expect #i
                             if para_token.to_lowercase() == "#i" {
                if let Some(token) = token_strings.pop()  {

                    //Expect haz
                    if token.to_lowercase() == "haz"
                    {
                        //Expect variable name
                        if let Some(next_token) = token_strings.pop()
                        {
                            let var_name = next_token; 

                            if let Some(next_token) = token_strings.pop()
                            {
                                //Expect #it
                                if next_token.to_lowercase() == "#it"
                                {
                                    if let Some(next_token_iz) = token_strings.pop()
                                    {
                                        //Expect iz
                                        if next_token_iz.to_lowercase() == "iz"
                                        {
                                            if let Some(value_token) = token_strings.pop()
                                            {
                                                //Expect Variable value
                                                let var_value = value_token; 

                                                if let Some(mkay_value) = token_strings.pop()
                                                {
                                                    //Expect mkay value
                                                    if mkay_value.to_lowercase() == "#mkay"
                                                    {
                                                        //Declare stack with VariableInfo using name and value
                                                        let variable_info = VariableInfo {
                                                            name: var_name,
                                                            value: Some(var_value),
                                                            line_defined: 0,
                                                        };

                                                        //Push the variable to the scope stack
                                                        scope_stack.push(variable_info);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // IF there is variable usage, expect #lemme
                if para_token.to_lowercase() == "#lemme"
                                            {
                                                if let Some(variable_lemme) = token_strings.pop()
                                                {
                                                    //Expect see after #lemme
                                                    if variable_lemme.to_lowercase() == "see"
                                                    {
                                                        //Expect variable_name after see
                                                        if let Some(variable_name) = token_strings.pop()
                                                        {
                                                            let variable = scope_stack.last().unwrap(); 

                                                            //Push the variable value to the html_string
                                                            let value = variable.value.clone(); 
                                                            html_string.push_str(" ");
                                                            html_string.push_str(&value.unwrap());
                                                        }
                                                        
                                                    }

                                                    continue;
                                                }
                                            }

                            //If there is maek tag, there can be a list as per the grammar inside paragraf
                            //expect #maek
                            if para_token.to_lowercase() == "#maek" {
                                if let Some(list_token) = token_strings.pop() {
                                    
                                    //append the ul tag if list is found
                                    if list_token.to_lowercase() == "list" {

                                        
                                        html_string.push_str("\n<ul>");

                                        //consume list elements 
                                        while let Some(list_elem_token) = token_strings.pop() {
                                        
                                        //End of list found, append it to html_string
                                            if list_elem_token.to_lowercase() == "#oic" {
                                                html_string.push_str("\n</ul>\n");
                                                break;
                                            }


                                            //If #gimmeh is found, it is list item
                                            if list_elem_token.to_lowercase() == "#gimmeh" {
                                                if let Some(item_token) = token_strings.pop() {

                                                    //Append li to html_string for list item
                                                    if item_token.to_lowercase() == "item" {
                                                        html_string.push_str("\n<li>");

                                                        //consume text tokes
                                                        while let Some(item_content_token) =
                                                                token_strings.pop()
                                                                
                                                        {


                                                            // If variable usage is found, expect lemme
                                                            if item_content_token.to_lowercase() == "#lemme"
                                            {
                                                // expect see
                                                if let Some(variable_lemme) = token_strings.pop()
                                                {
                                                    if variable_lemme.to_lowercase() == "see"
                                                    {
                                                        //Add the value of the variable from the scope stacl
                                                        if let Some(variable_name) = token_strings.pop()
                                                        {
                                                            let variable = scope_stack.last().unwrap(); 


                                                            //Push the value of the variable to the scope stack
                                                            let value = variable.value.clone(); 
                                                            html_string.push_str(" ");
                                                            html_string.push_str(&value.unwrap());
                                                        }
                                                        
                                                    }

                                                    continue;
                                                }
                                            }
                                                    //end of list item, append </li> tag, and push it to html string
                                                            if item_content_token
                                                                .to_lowercase()
                                                                == "#mkay"
                                                            {
                                                                html_string.push_str(
                                                                    "</li>\n",
                                                                );
                                                                break;
                                                            }
                                                            html_string.push_str(" ");
                                                            html_string.push_str(
                                                                &item_content_token,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            //IF there is gimmeh
                            if para_token.to_lowercase() == "#gimmeh" {
                                if let Some(para_elem_token) = token_strings.pop() {

                                    // if there is newline, expect newline and append <br/>
                                    if para_elem_token.to_lowercase() == "newline" {
                                        html_string.push_str("\n<br/>\n");
                                    }

                                    // if there is newline, expect soundz and append <audio controls>
                                    if para_elem_token.to_lowercase() == "soundz" {
                                        if let Some(address_token) = token_strings.pop() {
                                            html_string.push_str(&format!(
                                                "\n<audio controls>\n<source src=\"{}\" type=\"audio/mpeg\">",
                                                address_token
                                            ));
                                        }

                                        //Append URL address of audio
                                        if let  Some(address_token) = token_strings.pop() {

                                            //Append audio end tag
                                            if address_token.to_lowercase() == "#mkay" {
                                                html_string.push_str("</audio>\n");
                                            }
                            
                                        } 
                                    }
    
                                    
                                    //if the element found is video
                                    if para_elem_token.to_lowercase() == "vidz" 
                                    {
                                        //Append iframe src tag
                                        if let Some(address_token) = token_strings.pop() {
                                            html_string.push_str(&format!(
                                                "\n<iframe src = {}>\n",
                                                address_token
                                            ));
                                        }


                                        //Append the URL address of the video, consume till end of the tag is found
                                        if let  Some(address_token) = token_strings.pop() {
                                            if address_token.to_lowercase() == "#mkay" {
                                                continue;
                                            }
                            
                                        }
                                    }

                                    // else if the paragraf element found is bold, add starting bold tag
                                    if para_elem_token.to_lowercase() == "bold" {
                                        html_string.push_str(" <b>");

                                        //If variable usage is found, append the value of the variable
                                        while let Some(bold_token) = token_strings.pop() {
                                            //Expect #lemme
                                            if bold_token.to_lowercase() == "#lemme"
                                            {
                                                if let Some(variable_lemme) = token_strings.pop()
                                                {
                                                    //Expect see
                                                    if variable_lemme.to_lowercase() == "see"
                                                    {
                                                        if let Some(variable_name) = token_strings.pop()
                                                        {
                                                            //Find the value of variable and append it to the html string
                                                            let variable = scope_stack.last().unwrap(); 

                                                            let value = variable.value.clone(); 
                                                            html_string.push_str(" ");
                                                            html_string.push_str(&value.unwrap());
                                                        }
                                                        
                                                    }

                                                    continue;
                                                }
                                            }


                                            //end of bold input, append, ending bold tag
                                            if bold_token.to_lowercase() == "#mkay" {
                                                html_string.push_str(" </b>");
                                                break;
                                            }
                                            html_string.push_str(" ");
                                            html_string.push_str(&bold_token);
                                        }
                                    }

                                    //If the paragraf element found is italics, append starting italics tag
                                    if para_elem_token.to_lowercase() == "italics" {
                                        html_string.push_str(" <i>");
                                        while let Some(bold_token) = token_strings.pop() {

                                            //consume text tokens and append closing italics tag at the end
                                            if bold_token.to_lowercase() == "#mkay" {
                                                html_string.push_str(" </i>");
                                                break;
                                            }
                                            html_string.push_str(" ");
                                            html_string.push_str(&bold_token);
                                        }
                                    }



                                }

                                //If no matches found, consume all text elements (without #)
                            } else if !para_token.starts_with("#") {
                                html_string.push_str(" ");
                                html_string.push_str(&para_token);
                            }

                        
                    }

                }

                       
                }

                    
            }

            else  {

                //If the token found is variable usage, expect #lemme
                if token.to_lowercase() == "#lemme"
                                            {

                                                //Expect see
                                                if let Some(variable_lemme) = token_strings.pop()
                                                {
                                                    if variable_lemme.to_lowercase() == "see"
                                                    {

                                                        //Find the value of the variable, and append its value to the string
                                                        if let Some(variable_name) = token_strings.pop()
                                                        {
                                                            let variable = scope_stack.last().unwrap(); 

                                                            let value = variable.value.clone(); 
                                                            html_string.push_str(" ");
                                                            html_string.push_str(&value.unwrap());
                                                        }
                                                        
                                                    }

                                                    continue;
                                                }
                                            }

                                            // If the given token is gimmeh, 
               if token.to_lowercase() == "#gimmeh" {

                    //If there is newline tag, append <br> to the html string
                                if let Some(para_elem_token) = token_strings.pop() {
                                    if para_elem_token.to_lowercase() == "newline" {
                                        html_string.push_str("\n<br/>\n");
                                    }

                        //If there is soundz tag, append <audio controls> to the html string

                                    if para_elem_token.to_lowercase() == "soundz" {
                                        if let Some(address_token) = token_strings.pop() {
                                            html_string.push_str(&format!(
                                                "\n<audio controls>\n<source src=\"{}\" type=\"audio/mpeg\">",
                                                address_token
                                            ));
                                        }

                                        //consume URL address till end tag; append tag
                                        if let  Some(address_token) = token_strings.pop() {
                                            if address_token.to_lowercase() == "#mkay" {
                                                html_string.push_str("</audio>\n");
                                            }
                            
                                        } 
                                    }
    
                                        //If there is vidzoundz tag, append <iframe src> to the html string
                                    if para_elem_token.to_lowercase() == "vidz" 
                                    {
                                        if let Some(address_token) = token_strings.pop() {
                                            html_string.push_str(&format!(
                                                "\n<iframe src = {}>\n",
                                                address_token
                                            ));
                                        }

                                        //Get URL address
                                        if let  Some(address_token) = token_strings.pop() {
                                            if address_token.to_lowercase() == "#mkay" {
                                                continue;
                                            }
                            
                                        }
                                    }

                                    //If bold is found, consume bold elements
                                    if para_elem_token.to_lowercase() == "bold" {
                                        html_string.push_str(" <b>");
                                        while let Some(bold_token) = token_strings.pop() {

                                            // If variable usage, found expect #Lemme
                                            if bold_token.to_lowercase() == "#lemme"
                                            {
                                                if let Some(variable_lemme) = token_strings.pop()
                                                {
                                                    //Expect see
                                                    if variable_lemme.to_lowercase() == "see"
                                                    {
                                                        if let Some(variable_name) = token_strings.pop()
                                                        {
                                                            //Append the value of the variable to the value
                                                            let variable = scope_stack.last().unwrap(); 

                                                            let value = variable.value.clone(); 
                                                            html_string.push_str(" ");
                                                            html_string.push_str(&value.unwrap());
                                                        }
                                                        
                                                    }

                                                    continue;
                                                }
                                            }

                                            //Add the ending bold tag
                                            if bold_token.to_lowercase() == "#mkay" {
                                                html_string.push_str(" </b>");
                                                break;
                                            }
                                            html_string.push_str(" ");
                                            html_string.push_str(&bold_token);
                                        }
                                    }

                                    //If italics is found, append <i> tag

                                    if para_elem_token.to_lowercase() == "italics" {
                                        html_string.push_str(" <i>");

                                        //consume text tokens and append </i> tags
                                        while let Some(bold_token) = token_strings.pop() {
                                            if bold_token.to_lowercase() == "#mkay" {
                                                html_string.push_str(" </i>");
                                                break;
                                            }
                                            html_string.push_str(" ");
                                            html_string.push_str(&bold_token);
                                        }
                                    }



                                }

                            }

                //If any text tokens (non-tags) are found, push it to the html tokens
                else if !token.starts_with("#")
                {
                    html_string.push_str(" ");
                    html_string.push_str(&token);               
                }

                //for any tag keywords without hash-tags, skip them 
                else  {
                   continue;
                }

            }
        }

        //return html string
             html_string

    }

}


// Implementation for the LolCodeCompiler
impl Compiler for LolcodeCompiler {

    //method to start tokenization and getting first token
    fn compile(&mut self, source: &str) {

        //Initialize a lexer
        self.lexer = LolcodeLexicalAnalyzer::new(source);

        //Tokenize the lexer into tokens
        self.lexer.tokenize();

        //Get language tokens - used later for HTML conversion
        self.language_tokens = self.lexer.tokens.clone();

        //Get the first input token 
        self.start();
    }

    //method to lexically analyzer a token
    fn next_token(&mut self) -> String {

        //Pop a token
        let result = self.lexer.tokens.pop();


        //Return a lexeme and its line if it is valid, else through an error
        if let Some((candidate, line)) = result {
            self.parser.current_line = line;

            if self.lexer.lookup(&candidate) {
                self.current_tok = candidate.clone();
                candidate
            } else {
                eprintln!(
                    "Lexical error at line {}: '{}' is not a recognized token.",
                    line, candidate
                );
                std::process::exit(1);
            }
        } 
        //nothing found, clear current token and initialize new string
        else {
            self.current_tok.clear();
            String::new()
        }
    }

    // Start parsing lolcode
    fn parse(&mut self) {

        //Call lolcode method to start parsing lolcode
        self.lolcode();

        //If no input found, report an error
        if !self.lexer.tokens.is_empty() {
            eprintln!(
                "Syntax error at line {}: Additional tokens found after the document.",
                self.parser.current_line
            );
            std::process::exit(1);
        }
    }

    //Return the clone of current token
    fn current_token(&self) -> String {
        self.current_tok.clone()
    }

    //Set the current token of the compiler
    fn set_current_token(&mut self, tok: String) {
        self.current_tok = tok;
    }
}

//Custom class to validate a file path or report an error, includes a file path
struct Config {
    file_path: String,
}

//implementation for Config
impl Config {

    //Report an error if no file path is found
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments, add a file argument");
        }

        //return file path from second argument
        let file_path = args[1].clone();

        //file_path validated, returns OK
        Ok(Config { file_path })
    }
}


//Function to open chrome in html
pub fn open_html_in_chrome<P: AsRef<Path>>(html_file: P) -> io::Result<()> {

    //Get the reference of file
    let p = html_file.as_ref();
    

    //If path does not exist,  report error
    if !p.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", p.display()),
        ));
    }
    
    //Cannonicalize path to URL 
    let abs = fs::canonicalize(p)?;
    
    // Handle potential non-UTF8 paths gracefully
    let path_str = abs.to_str().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Path contains invalid UTF-8")
    })?;
    

    //Get the clean path
    let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);
    
    // Convert to file:// URL for chrome display
    let file_url = format!("file:///{}", clean_path.replace('\\', "/"));
    
    
    // Try to find Chrome from registry if not defined in path
    if let Some(chrome_path) = find_chrome_path() {
        return Command::new(chrome_path)
            .arg(&file_url)
            .spawn()
            .map(|_| ())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
    }
    
    // Fallback to 'start chrome' command
    let status = Command::new("cmd")
        .args(&["/C", "start", "chrome", &file_url])
        .status()?;

    
    //Return success or error if chrome is not launced
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Could not find or launch Chrome",
        ))
    }
}


//Find chrome path in system 
#[cfg(windows)]
fn find_chrome_path() -> Option<String> {
    // Registry keys where Chrome might be registered
    let registry_paths = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe",
        r"SOFTWARE\Google\Chrome\BLBeacon",
    ];
    
    //predefinition of how registry will look like
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    
    // Try each registry path, and associated keys, till a path where chrome exists is found
    for reg_path in &registry_paths {
        if let Ok(key) = hklm.open_subkey(reg_path) {
            // Try to read the default value or "Path" value
            if let Ok(path) = key.get_value::<String, _>("") {
                if Path::new(&path).exists() {
                    return Some(path);
                }
            }
            
            // Some keys store it in a "version" subkey
            if let Ok(path) = key.get_value::<String, _>("Path") {
                let chrome_exe = format!("{}\\chrome.exe", path);
                if Path::new(&chrome_exe).exists() {
                    return Some(chrome_exe);
                }
            }
        }
    }
    
    // Also try HKEY_CURRENT_USER by parsing throuhg all the paths
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for reg_path in &registry_paths {
        if let Ok(key) = hkcu.open_subkey(reg_path) {
            if let Ok(path) = key.get_value::<String, _>("") {
                if Path::new(&path).exists() {
                    return Some(path);
                }
            }
        }
    }
    
    None
}


fn main() {

    //Collect all file arguments
    let args: Vec<String> = env::args().collect();

    //Report error if problem parsing arguments
    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

// Validate .lol extension
let file_path = Path::new(&config.file_path);
match file_path.extension().and_then(|ext| ext.to_str()) {
    Some("lol") => {
        //Continue
    }
    //Error - wrong file extension
    Some(other) => {
        println!("Error: Invalid file extension '.{}'. Only .lol files are accepted.", other);
        process::exit(1);
    }
    //Extension not found
    None => {
        println!("Error: No file extension found. Only .lol files are accepted.");
        process::exit(1);
    }
}

//Initialize html file at file path based on first name of .lol file in the same location
let html_filename = file_path
.file_stem()
.and_then(|name| name.to_str())
.map(|name| format!("{}.html", name))
.unwrap_or_else(|| "output.html".to_string()); 

//Read string from file and set into lolcode string
    let lolcode_string: String;
    match read_to_string(config.file_path) {
        Ok(contents) => lolcode_string = contents,

        //Report an error if not able to read file
        Err(e) => {
            println!("Error reading the file: {e}");
            process::exit(1);
        }
    }

    //Initialize a compiler
    let mut compiler = LolcodeCompiler::new();

    //Compile the file
    compiler.compile(&lolcode_string);

    //Parse the file
    compiler.parse();


    //Get the html string from file conversion and parsing
    let html_string: String = compiler.to_html();


    //Write the html to the file 
    std::fs::write(&html_filename, html_string).expect("Unable to write file"); 

    //open the file in html
    open_html_in_chrome(&html_filename); 
    
   
}