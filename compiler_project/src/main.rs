
use std::{env, process, vec}; 
use std::fs::read_to_string; 
use regex::Regex; 

pub struct LolcodeCompiler{
    lexer: LolcodeLexicalAnalyzer,
    parser: LolcodeSyntaxAnalyzer, 
    current_tok: String, 
}

pub trait Compiler {
    fn compile(&mut self, source: &str);
    fn next_token(&mut self) -> String;
    fn parse(&mut self);
    fn current_token(&self) -> String;
    fn set_current_token(&mut self, tok: String);
}

pub trait  LexicalAnalyzer {
    fn get_char(&mut self) -> char; 
    fn add_char(&mut self) -> char; 
    fn lookup(&self, s: &str) -> bool; 
} 

pub struct LolcodeLexicalAnalyzer{
    input: Vec<char>, 
    position: usize, 
    current_build: String,
    tokens: Vec<String>, 
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

impl LolcodeLexicalAnalyzer {
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            position: 0, 
            current_build: String::new(),
            tokens: Vec::new(),
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
            var_def : Regex::new(r"^[A-Za-z]+$").unwrap(),
            var_val: Regex::new(r"^[A-Za-z0-9,\.\':\?!_\/ ]+$").unwrap(),
            text : Regex::new(r"^[A-Za-z0-9,\.\':\?!_\/ ]+$").unwrap(),
            address: Regex::new(r"^[A-Za-z0-9,\.\':\?!_\/%]+$").unwrap(),
        }
    }

    pub fn tokenize(&mut self) {
        loop {
            let c = self.get_char(); 

            if c == '\0' {
                break; 
            }

            if c.is_whitespace() {
                if !self.current_build.is_empty() {
                    self.tokens.push(std::mem::take(&mut self.current_build));
                }
            } else {
                self.add_char(); 
            }
        }

        if !self.current_build.is_empty() {
            self.tokens.push(std::mem::take(&mut self.current_build)); 
        }
        
        // Reverse to get first token when popping
        self.tokens.reverse();

       
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

    fn lookup(&self, s: &str) -> bool {
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
        
        self.head_element.iter().any(|h| h == &s.to_lowercase()) 
            || self.title_element.iter().any(|h| h == &s.to_lowercase()) 
            || self.paragraph_element.iter().any(|h| h == &s.to_lowercase()) 
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

// Parser - syntax rules 
pub trait SyntaxAnalyzer {
    fn parse_lolcode(&mut self, compiler: &mut LolcodeCompiler); 
    fn parse_head(&mut self, compiler: &mut LolcodeCompiler);           
    fn parse_title(&mut self, compiler: &mut LolcodeCompiler);         
    fn parse_comment(&mut self, compiler: &mut LolcodeCompiler);        
    fn parse_body(&mut self, compiler: &mut LolcodeCompiler);           
    fn parse_paragraph(&mut self, compiler: &mut LolcodeCompiler);      
    fn parse_list(&mut self, compiler: &mut LolcodeCompiler);           
    fn parse_item(&mut self, compiler: &mut LolcodeCompiler);     
    fn parse_audio(&mut self, compiler: &mut LolcodeCompiler);          
    fn parse_video(&mut self, compiler: &mut LolcodeCompiler);          
    fn parse_newline(&mut self, compiler: &mut LolcodeCompiler);        
    fn parse_bold(&mut self, compiler: &mut LolcodeCompiler);           
    fn parse_italics(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_text(&mut self, compiler: &mut LolcodeCompiler);
}

pub struct LolcodeSyntaxAnalyzer {
}

impl LolcodeSyntaxAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
    
    // Helper methods to check token types using compiler's lexer
    fn is_document_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.head_start.iter().any(|head| head == &s.to_lowercase())
    }

    fn is_document_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.head_end.iter().any(|head| head == &s.to_lowercase())
    }

    fn is_make_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.make_start.iter().any(|make| make == &s.to_lowercase())
    }

    fn is_oic_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.oic_end.iter().any(|oic| oic == &s.to_lowercase())
    }

    fn is_gimmeh_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.gimmeh_start.iter().any(|gimmeh| gimmeh == &s.to_lowercase())
    }

    fn is_mkay_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.mkay_end.iter().any(|mkay| mkay == &s.to_lowercase())
    }

    fn is_comment_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.comment_start.iter().any(|comment| comment == &s.to_lowercase())
    }

    fn is_comment_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.comment_end.iter().any(|comment| comment == &s.to_lowercase())
    }

    fn is_head_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.head_element.iter().any(|head| head == &s.to_lowercase())
    }

    fn is_title_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.title_element.iter().any(|title| title == &s.to_lowercase())
    }

    fn is_paragraph_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.paragraph_element.iter().any(|para| para == &s.to_lowercase())
    }

    fn is_bold_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.bold_element.iter().any(|bold| bold == &s.to_lowercase())
    }

    fn is_italics_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.italics_element.iter().any(|italic| italic == &s.to_lowercase())
    }

    fn is_list_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.list_element.iter().any(|list| list == &s.to_lowercase())
    }

    fn is_item_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.item_element.iter().any(|item| item == &s.to_lowercase())
    }

    fn is_newline_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.newline_element.iter().any(|nl| nl == &s.to_lowercase())
    }

    fn is_soundz_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.soundz_element.iter().any(|sound| sound == &s.to_lowercase())
    }

    fn is_vidz_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.vidz_element.iter().any(|vid| vid == &s.to_lowercase())
    }

    fn is_text(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.text.is_match(s)
    }

    fn is_address(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.address.is_match(s)
    }

    
}

impl SyntaxAnalyzer for LolcodeSyntaxAnalyzer {
    fn parse_lolcode(&mut self, compiler: &mut LolcodeCompiler) {
        
        // Optional comment at start
        if self.is_comment_start(&compiler.current_tok, &compiler.lexer) {
            self.parse_comment(compiler);
        }
        
        // Parse head (required)
        self.parse_head(compiler);
        
        // Parse body (required)
        self.parse_body(compiler);
    }

    fn parse_head(&mut self, compiler: &mut LolcodeCompiler) {
        // Expect #MAEK
        if !self.is_make_start(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected '#maek', found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
        
        // Expect HEAD
        if !self.is_head_element(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected 'head', found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
        
        // Parse title
        self.parse_title(compiler);
        
        // Expect #OIC
        if !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected '#oic', found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
    }

    fn parse_title(&mut self, compiler: &mut LolcodeCompiler) {
        // Expect #GIMMEH
        if !self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected '#gimmeh', found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
        
        // Expect TITLE
        if !self.is_title_element(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected 'title', found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
        
        // Consume text until #MKAY
        while !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!("Syntax error: Unexpected end of input in title.");
                std::process::exit(1);
            }
            self.parse_text(compiler);
        }
        
        // Consume #MKAY
        compiler.current_tok = compiler.next_token();
    }

    fn parse_comment(&mut self, compiler: &mut LolcodeCompiler) {
        // Expect #OBTW
        if !self.is_comment_start(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected comment start '#obtw', found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
        
        // Consume all text until #TLDR
        while !self.is_comment_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!("Syntax error: Unexpected end of input in comment.");
                std::process::exit(1);
            }
            compiler.current_tok = compiler.next_token();
        }
        
        // Consume #TLDR
        compiler.current_tok = compiler.next_token();
    }

    fn parse_body(&mut self, compiler: &mut LolcodeCompiler) {
        // Parse body elements until we hit #KTHXBYE
        while !self.is_document_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!("Syntax error: Unexpected end of input in body.");
                std::process::exit(1);
            }
            
            if self.is_make_start(&compiler.current_tok, &compiler.lexer) {
                compiler.current_tok = compiler.next_token();
                
                if self.is_paragraph_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_paragraph(compiler);
                } else if self.is_list_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_list(compiler);
                } else if self.is_bold_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_bold(compiler);
                } else if self.is_italics_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_italics(compiler);
                } else {
                    eprintln!("Syntax error: Unknown element after '#maek': '{}'.", compiler.current_tok);
                    std::process::exit(1);
                }
            } else if self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {
                compiler.current_tok = compiler.next_token();
                
                if self.is_newline_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_newline(compiler);
                } else if self.is_soundz_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_audio(compiler);
                } else if self.is_vidz_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_video(compiler);
                } else {
                    eprintln!("Syntax error: Unknown element after '#gimmeh': '{}'.", compiler.current_tok);
                    std::process::exit(1);
                }
            } else if self.is_comment_start(&compiler.current_tok, &compiler.lexer) {
                self.parse_comment(compiler);
            } else if self.is_text(&compiler.current_tok, &compiler.lexer) {
                self.parse_text(compiler);
            } else {
                eprintln!("Syntax error: Unexpected token in body: '{}'.", compiler.current_tok);
                std::process::exit(1);
            }
        }
    }

    fn parse_paragraph(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #MAEK PARAGRAF
        compiler.current_tok = compiler.next_token();
        
        // Parse paragraph content until #OIC
        while !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!("Syntax error: Unexpected end of input in paragraph.");
                std::process::exit(1);
            }
            
            if self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {
                compiler.current_tok = compiler.next_token();
                
                if self.is_newline_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_newline(compiler);
                } else if self.is_soundz_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_audio(compiler);
                } else if self.is_vidz_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_video(compiler);
                } else if self.is_bold_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_bold(compiler);
                } else {
                    eprintln!("Syntax error: Unknown element after '#gimmeh': '{}'.", compiler.current_tok);
                    std::process::exit(1);
                }
            } else if self.is_make_start(&compiler.current_tok, &compiler.lexer) {
                compiler.current_tok = compiler.next_token();
                
                if self.is_list_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_list(compiler);
                } else if self.is_bold_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_bold(compiler);
                } else {
                    eprintln!("Syntax error: Unknown element after '#maek': '{}'.", compiler.current_tok);
                    std::process::exit(1);
                }
            } else if self.is_text(&compiler.current_tok, &compiler.lexer) {
                self.parse_text(compiler);
            } else {
                eprintln!("Syntax error: Unexpected token in paragraph: '{}'.", compiler.current_tok);
                std::process::exit(1);
            }
        }
        
        // Consume #OIC
        compiler.current_tok = compiler.next_token();
    }

    fn parse_list(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #MAEK LIST
        compiler.current_tok = compiler.next_token();
        
        // Parse list items until #OIC
        while !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!("Syntax error: Unexpected end of input in list.");
                std::process::exit(1);
            }
            
            if self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {
                compiler.current_tok = compiler.next_token();
                
                if self.is_item_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_item(compiler);
                } else {
                    eprintln!("Syntax error: Expected 'item' after '#gimmeh', found '{}'.", compiler.current_tok);
                    std::process::exit(1);
                }
            } else {
                eprintln!("Syntax error: Expected '#gimmeh item' in list, found '{}'.", compiler.current_tok);
                std::process::exit(1);
            }
        }
        
        // Consume #OIC
        compiler.current_tok = compiler.next_token();
    }

    fn parse_item(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH ITEM
        compiler.current_tok = compiler.next_token();
        
        // Parse item content until #MKAY
        while !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!("Syntax error: Unexpected end of input in list item.");
                std::process::exit(1);
            }
            self.parse_text(compiler);
        }
        
        // Consume #MKAY
        compiler.current_tok = compiler.next_token();
    }

    fn parse_audio(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH SOUNDZ
        compiler.current_tok = compiler.next_token();
        
        // Expect address
        if !self.is_address(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected address for audio, found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
        
        // Expect #MKAY
        if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected '#mkay' after audio address, found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
    }

    fn parse_video(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH VIDZ
        compiler.current_tok = compiler.next_token();
        
        // Expect address
        if !self.is_address(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected address for video, found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
        
        // Expect #MKAY
        if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected '#mkay' after video address, found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
    }

    fn parse_newline(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH NEWLINE
        compiler.current_tok = compiler.next_token();
    }

    fn parse_bold(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH BOLD or #MAEK BOLD (check context)
        compiler.current_tok = compiler.next_token();
        
        // Parse text until #MKAY
        while !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!("Syntax error: Unexpected end of input in bold.");
                std::process::exit(1);
            }
            self.parse_text(compiler);
        }
        
        // Consume #MKAY
        compiler.current_tok = compiler.next_token();
    }

    fn parse_italics(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #MAEK ITALICS
        compiler.current_tok = compiler.next_token();
        
        // Parse text until #OIC
        while !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!("Syntax error: Unexpected end of input in italics.");
                std::process::exit(1);
            }
            self.parse_text(compiler);
        }
        
        // Consume #OIC
        compiler.current_tok = compiler.next_token();
    }

    fn parse_text(&mut self, compiler: &mut LolcodeCompiler) {
        if !self.is_text(&compiler.current_tok, &compiler.lexer) {
            eprintln!("Syntax error: Expected text, found '{}'.", compiler.current_tok);
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
    }
}

impl LolcodeCompiler {
    pub fn new() -> Self {
        Self {
            lexer: LolcodeLexicalAnalyzer::new(""),
            parser: LolcodeSyntaxAnalyzer::new(),
            current_tok: String::new(), 
        }
    }

    fn start(&mut self) {
        // Get the first token
        self.current_tok = self.next_token();
        
        if self.current_tok.is_empty() {
            eprintln!("User error: The provided sentence is empty.");
            std::process::exit(1);
        }
    }

    fn lolcode(&mut self) {
        // Document should start with #HAI
        if !self.lexer.head_start.iter().any(|h| h == &self.current_tok.to_lowercase()) {
            eprintln!("Syntax error: Expected document start '#hai', found '{}'.", self.current_tok);
            std::process::exit(1);
        }
        
        self.current_tok = self.next_token();
        
        // Parse the document structure
        // Need to temporarily move parser out to avoid borrow conflict
        let mut parser = std::mem::replace(&mut self.parser, LolcodeSyntaxAnalyzer::new());
        parser.parse_lolcode(self);
        self.parser = parser;
        
        // Document should end with #KTHXBYE
        if !self.lexer.head_end.iter().any(|h| h == &self.current_tok.to_lowercase()) {
            eprintln!("Syntax error: Expected document end '#kthxbye', found '{}'.", self.current_tok);
            std::process::exit(1);
        }
    }
}

impl Compiler for LolcodeCompiler {
    fn compile(&mut self, source: &str) {
        self.lexer = LolcodeLexicalAnalyzer::new(source);
        self.lexer.tokenize();
        self.start();
    }

    fn next_token(&mut self) -> String {
       let candidate= self.lexer.tokens.pop().unwrap_or_default(); 
       if self.lexer.lookup(&candidate) {
            self.current_tok = candidate.clone();
            candidate
        } else if self.lexer.tokens.is_empty() {
            self.current_tok.clear();
            String::new()
        } else {
            eprintln!("Lexical error: '{}' is not a recognized token.", candidate);
            std::process::exit(1);
        }
    }

    fn parse(&mut self) {
        self.lolcode();
        
        if !self.lexer.tokens.is_empty() {
            eprintln!("Syntax error: Additional tokens found after the document.");
            std::process::exit(1);
        }
    }

    fn current_token(&self) -> String {
        self.current_tok.clone()
    }

    fn set_current_token(&mut self, tok: String) {
        self.current_tok = tok;
    }
}

struct Config {
    file_path: String, 
}

impl Config {
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
    compiler.parse();
    
    println!("This lolcode script is syntactically valid.");
}