use regex::Regex;
use std::collections::HashMap;
use std::fs::{File, read_to_string};
use std::{env, process, vec, io};
use std::{fs, path::Path, process::Command};

pub struct LolcodeCompiler {
    lexer: LolcodeLexicalAnalyzer,
    parser: LolcodeSyntaxAnalyzer,
    current_tok: String,
    scope_stack: Vec<HashMap<String, VariableInfo>>,
    language_tokens: Vec<(String, usize)>,
}

#[derive(Clone)]
struct VariableInfo {
    name: String,
    value: Option<String>,
    line_defined: usize,
}

pub trait Compiler {
    fn compile(&mut self, source: &str);
    fn next_token(&mut self) -> String;
    fn parse(&mut self);
    fn current_token(&self) -> String;
    fn set_current_token(&mut self, tok: String);
}

pub trait LexicalAnalyzer {
    fn get_char(&mut self) -> char;
    fn add_char(&mut self) -> char;
    fn lookup(&self, s: &str) -> bool;
}

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

    pub fn tokenize(&mut self) {
        loop {
            let c = self.get_char();

            if c == '\0' {
                break;
            }

            if c == '\n' {
                if !self.current_build.is_empty() {
                    self.tokens
                        .push((std::mem::take(&mut self.current_build), self.line_number));
                }
                self.line_number += 1;
            } else if c.is_whitespace() {
                if !self.current_build.is_empty() {
                    self.tokens
                        .push((std::mem::take(&mut self.current_build), self.line_number));
                }
            } else {
                self.add_char();
            }
        }

        if !self.current_build.is_empty() {
            self.tokens
                .push((std::mem::take(&mut self.current_build), self.line_number));
        }

        // Reverse to get first token when popping
        self.tokens.reverse();
    }

    pub fn return_tokens(&mut self) {
        self.tokens.clone();
    }
    fn is_variable_identifier(&self, s: &str) -> bool {
        self.var_def.is_match(s)
    }
}

impl LexicalAnalyzer for LolcodeLexicalAnalyzer {
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
    fn parse_variable_declaration(&mut self, compiler: &mut LolcodeCompiler);
    fn parse_variable_usage(&mut self, compiler: &mut LolcodeCompiler);
}

pub struct LolcodeSyntaxAnalyzer {
    current_line: usize,
}

impl LolcodeSyntaxAnalyzer {
    pub fn new() -> Self {
        Self { current_line: 1 }
    }

    // Helper methods to check token types using compiler's lexer

    fn is_document_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.head_end.iter().any(|head| head == &s.to_lowercase())
    }

    fn is_make_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .make_start
            .iter()
            .any(|make| make == &s.to_lowercase())
    }

    fn is_oic_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.oic_end.iter().any(|oic| oic == &s.to_lowercase())
    }

    fn is_gimmeh_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .gimmeh_start
            .iter()
            .any(|gimmeh| gimmeh == &s.to_lowercase())
    }

    fn is_mkay_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.mkay_end.iter().any(|mkay| mkay == &s.to_lowercase())
    }

    fn is_comment_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .comment_start
            .iter()
            .any(|comment| comment == &s.to_lowercase())
    }

    fn is_comment_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .comment_end
            .iter()
            .any(|comment| comment == &s.to_lowercase())
    }

    fn is_variable_start(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.variable_start.iter().any(|v| v == &s.to_lowercase())
    }

    fn is_variable_mid(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.variable_mid.iter().any(|v| v == &s.to_lowercase())
    }

    fn is_variable_end(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.variable_end.iter().any(|v| v == &s.to_lowercase())
    }

    fn is_head_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .head_element
            .iter()
            .any(|head| head == &s.to_lowercase())
    }

    fn is_title_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .title_element
            .iter()
            .any(|title| title == &s.to_lowercase())
    }

    fn is_paragraph_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .paragraph_element
            .iter()
            .any(|para| para == &s.to_lowercase())
    }

    fn is_bold_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .bold_element
            .iter()
            .any(|bold| bold == &s.to_lowercase())
    }

    fn is_italics_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .italics_element
            .iter()
            .any(|italic| italic == &s.to_lowercase())
    }

    fn is_list_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .list_element
            .iter()
            .any(|list| list == &s.to_lowercase())
    }

    fn is_item_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .item_element
            .iter()
            .any(|item| item == &s.to_lowercase())
    }

    fn is_newline_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .newline_element
            .iter()
            .any(|nl| nl == &s.to_lowercase())
    }

    fn is_soundz_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .soundz_element
            .iter()
            .any(|sound| sound == &s.to_lowercase())
    }

    fn is_vidz_element(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer
            .vidz_element
            .iter()
            .any(|vid| vid == &s.to_lowercase())
    }

    fn is_text(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.text.is_match(s)
    }

    fn is_address(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.address.is_match(s)
    }

    fn is_variable_identifier(&self, s: &str, lexer: &LolcodeLexicalAnalyzer) -> bool {
        lexer.is_variable_identifier(s)
    }
}

impl SyntaxAnalyzer for LolcodeSyntaxAnalyzer {
    fn parse_lolcode(&mut self, compiler: &mut LolcodeCompiler) {
        // Optional comment at start
        if self.is_comment_start(&compiler.current_tok, &compiler.lexer) {
            self.parse_comment(compiler);
        }

        // Allow variable declarations before head
        while self.is_variable_start(&compiler.current_tok, &compiler.lexer) {
            self.parse_variable_declaration(compiler);
        }

        // Parse head (optional) - only if we see #MAEK followed by HEAD
        if self.is_make_start(&compiler.current_tok, &compiler.lexer)
            && !compiler.lexer.tokens.is_empty()
            && self.is_head_element(&compiler.lexer.tokens.last().unwrap().0, &compiler.lexer)
        {
            self.parse_head(compiler);
        }

        // Parse body (required)
        self.parse_body(compiler);
    }

    fn parse_head(&mut self, compiler: &mut LolcodeCompiler) {
        // Expect #MAEK
        if !self.is_make_start(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#maek', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();

        // Expect HEAD
        if !self.is_head_element(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected 'head', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();

        // Parse title
        self.parse_title(compiler);

        // Expect #OIC
        if !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#oic', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
    }

    fn parse_title(&mut self, compiler: &mut LolcodeCompiler) {
        // Expect #GIMMEH
        if !self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#gimmeh', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();

        // Expect TITLE
        if !self.is_title_element(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected 'title', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();

        // Consume text until #MKAY
        while !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in title.",
                    self.current_line
                );
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
            eprintln!(
                "Syntax error at line {}: Expected comment start '#obtw', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();

        // Consume all text until #TLDR
        while !self.is_comment_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in comment.",
                    self.current_line
                );
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
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in body.",
                    self.current_line
                );
                std::process::exit(1);
            }

            if self.is_make_start(&compiler.current_tok, &compiler.lexer) {
                compiler.current_tok = compiler.next_token();

                if self.is_paragraph_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_paragraph(compiler);
                } else if self.is_list_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_list(compiler);
                } else {
                    eprintln!(
                        "Syntax error at line {}: Unknown element after '#maek': '{}'.",
                        self.current_line, compiler.current_tok
                    );
                    std::process::exit(1);
                }
            } else if self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {
                compiler.current_tok = compiler.next_token();

                if self.is_newline_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_newline(compiler);
                } 
                else if self.is_bold_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_bold(compiler);
                } else if self.is_italics_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_italics(compiler);
                } else if self.is_soundz_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_audio(compiler);
                } else if self.is_vidz_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_video(compiler);
                } else {
                    eprintln!(
                        "Syntax error at line {}: Unknown element after '#gimmeh': '{}'.",
                        self.current_line, compiler.current_tok
                    );
                    std::process::exit(1);
                }
            } else if self.is_variable_start(&compiler.current_tok, &compiler.lexer) {
                self.parse_variable_declaration(compiler);
            } else if self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
                self.parse_variable_usage(compiler);
            } else if self.is_comment_start(&compiler.current_tok, &compiler.lexer) {
                self.parse_comment(compiler);
            } else if self.is_text(&compiler.current_tok, &compiler.lexer) {
                self.parse_text(compiler);
            } else {
                eprintln!(
                    "Syntax error at line {}: Unexpected token in body: '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }
        }
    }

    fn parse_paragraph(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #MAEK PARAGRAF
        compiler.current_tok = compiler.next_token();

        // PUSH NEW SCOPE when entering paragraph
        compiler.push_scope();

        // Parse paragraph content until #OIC
        while !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in paragraph.",
                    self.current_line
                );
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
                }else if self.is_italics_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_italics(compiler);
                } 
                else {
                    eprintln!(
                        "Syntax error at line {}: Unknown element after '#gimmeh': '{}'.",
                        self.current_line, compiler.current_tok
                    );
                    std::process::exit(1);
                }
            } else if self.is_make_start(&compiler.current_tok, &compiler.lexer) {
                compiler.current_tok = compiler.next_token();

                if self.is_list_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_list(compiler);
                } else {
                    eprintln!(
                        "Syntax error at line {}: Unknown element after '#maek': '{}'.",
                        self.current_line, compiler.current_tok
                    );
                    std::process::exit(1);
                }
            } else if self.is_variable_start(&compiler.current_tok, &compiler.lexer) {
                self.parse_variable_declaration(compiler);
            } else if self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
                self.parse_variable_usage(compiler);
            } else if self.is_text(&compiler.current_tok, &compiler.lexer) {
                self.parse_text(compiler);
            } else {
                eprintln!(
                    "Syntax error at line {}: Unexpected token in paragraph: '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }
        }

        // Consume #OIC
        compiler.current_tok = compiler.next_token();

        // POP SCOPE when exiting paragraph
        compiler.pop_scope();
    }

    fn parse_list(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #MAEK LIST
        compiler.current_tok = compiler.next_token();

        // Parse list items until #OIC
        while !self.is_oic_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in list.",
                    self.current_line
                );
                std::process::exit(1);
            }

            if self.is_gimmeh_start(&compiler.current_tok, &compiler.lexer) {
                compiler.current_tok = compiler.next_token();

                if self.is_item_element(&compiler.current_tok, &compiler.lexer) {
                    self.parse_item(compiler);
                } else {
                    eprintln!(
                        "Syntax error at line {}: Expected 'item' after '#gimmeh', found '{}'.",
                        self.current_line, compiler.current_tok
                    );
                    std::process::exit(1);
                }
            } else {
                eprintln!(
                    "Syntax error at line {}: Expected '#gimmeh item' in list, found '{}'.",
                    self.current_line, compiler.current_tok
                );
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
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in list item.",
                    self.current_line
                );
                std::process::exit(1);
            }

            if self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
                self.parse_variable_usage(compiler);
            } else {
                self.parse_text(compiler);
            }
        }

        // Consume #MKAY
        compiler.current_tok = compiler.next_token();
    }

    fn parse_audio(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH SOUNDZ
        compiler.current_tok = compiler.next_token();

        // Expect address
        if !self.is_address(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected address for audio, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();

        // Expect #MKAY
        if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#mkay' after audio address, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
    }

    fn parse_video(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH VIDZ
        compiler.current_tok = compiler.next_token();

        // Expect address
        if !self.is_address(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected address for video, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();

        // Expect #MKAY
        if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#mkay' after video address, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
    }

    fn parse_newline(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH NEWLINE
        compiler.current_tok = compiler.next_token();
    }

    fn parse_bold(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #GIMMEH BOLD
        compiler.current_tok = compiler.next_token();

        // Parse text until #MKAY
        while !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in bold.",
                    self.current_line
                );
                std::process::exit(1);
            }

            if self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
                self.parse_variable_usage(compiler);
            } else {
                self.parse_text(compiler);
            }
        }

        // Consume #MKAY
        compiler.current_tok = compiler.next_token();
    }

    fn parse_italics(&mut self, compiler: &mut LolcodeCompiler) {
        // Already consumed #MAEK ITALICS
        compiler.current_tok = compiler.next_token();

        // Parse text until #OIC
        while !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            if compiler.current_tok.is_empty() {
                eprintln!(
                    "Syntax error at line {}: Unexpected end of input in italics.",
                    self.current_line
                );
                std::process::exit(1);
            }

            if self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
                self.parse_variable_usage(compiler);
            } else {
                self.parse_text(compiler);
            }
        }

        // Consume #OIC
        compiler.current_tok = compiler.next_token();
    }

    fn parse_text(&mut self, compiler: &mut LolcodeCompiler) {
        if !self.is_text(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected text, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }
        compiler.current_tok = compiler.next_token();
    }

    fn parse_variable_declaration(&mut self, compiler: &mut LolcodeCompiler) {
        // Expect #I HAZ
        if !self.is_variable_start(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#i' or 'haz', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        let var_keyword = compiler.current_tok.to_lowercase();
        compiler.current_tok = compiler.next_token();

        // If we saw #I, expect HAZ
        if var_keyword == "#i" {
            if compiler.current_tok.to_lowercase() != "haz" {
                eprintln!(
                    "Syntax error at line {}: Expected 'haz' after '#i', found '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }
            compiler.current_tok = compiler.next_token();
        }

        // Expect variable identifier
        if !self.is_variable_identifier(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected variable identifier, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        let var_name = compiler.current_tok.clone();
        compiler.current_tok = compiler.next_token();

        // Check for #IT IZ (value assignment)
        let var_value = if self.is_variable_mid(&compiler.current_tok, &compiler.lexer) {
            let mid_keyword = compiler.current_tok.to_lowercase();
            compiler.current_tok = compiler.next_token();

            // If we saw #IT, expect IZ
            if mid_keyword == "#it" {
                if compiler.current_tok.to_lowercase() != "iz" {
                    eprintln!(
                        "Syntax error at line {}: Expected 'iz' after '#it', found '{}'.",
                        self.current_line, compiler.current_tok
                    );
                    std::process::exit(1);
                }
                compiler.current_tok = compiler.next_token();
            }

            // Expect value (text)
            if !self.is_text(&compiler.current_tok, &compiler.lexer)
                && !self.is_address(&compiler.current_tok, &compiler.lexer)
            {
                eprintln!(
                    "Syntax error at line {}: Expected value after 'iz', found '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }
            let value = compiler.current_tok.clone();
            compiler.current_tok = compiler.next_token();

            if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
                eprintln!(
                    "Syntax error at line {}: Expected '#mkay' after variable value, found '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }
            //Add statement for mkay

            compiler.current_tok = compiler.next_token();

            Some(value)
        } else {
            None
        };

        compiler.declare_variable(var_name, var_value, self.current_line);
    }

    fn parse_variable_usage(&mut self, compiler: &mut LolcodeCompiler) {
        // Expect #LEMME SEE
        if !self.is_variable_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#lemme' or 'see', found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        let var_keyword = compiler.current_tok.to_lowercase();
        compiler.current_tok = compiler.next_token();

        // If we saw #LEMME, expect SEE
        if var_keyword == "#lemme" {
            if compiler.current_tok.to_lowercase() != "see" {
                eprintln!(
                    "Syntax error at line {}: Expected 'see' after '#lemme', found '{}'.",
                    self.current_line, compiler.current_tok
                );
                std::process::exit(1);
            }
            compiler.current_tok = compiler.next_token();
        }

        // Expect variable identifier
        if !self.is_variable_identifier(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected variable identifier, found '{}'.",
                self.current_line, compiler.current_tok
            );
            std::process::exit(1);
        }

        let var_name = compiler.current_tok.clone();

        // Check if variable is defined using lookup_variable
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

        compiler.current_tok = compiler.next_token();

        if !self.is_mkay_end(&compiler.current_tok, &compiler.lexer) {
            eprintln!(
                "Syntax error at line {}: Expected '#mkay' after variable usage, found '{}'.",
                self.current_line, compiler.current_tok
            );
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
            scope_stack: vec![HashMap::new()],
            language_tokens: vec![],
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

        self.current_tok = self.next_token();

        // Parse the document structure
        let mut parser = std::mem::replace(&mut self.parser, LolcodeSyntaxAnalyzer::new());
        parser.parse_lolcode(self);
        self.parser = parser;

        // Document should end with #KTHXBYE
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

    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    fn declare_variable(&mut self, name: String, value: Option<String>, line: usize) {
        // Only check for redeclaration in the CURRENT scope
        if let Some(current_scope) = self.scope_stack.last_mut() {
            if current_scope.contains_key(&name) {
                let existing = &current_scope[&name];
                eprintln!(
                    "Semantic error at line {}: Variable '{}' is already defined at line {} in the current scope.",
                    line, name, existing.line_defined
                );
                std::process::exit(1);
            }

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

    fn lookup_variable(&self, name: &str) -> Option<&VariableInfo> {
        // Search from innermost to outermost scope
        for scope in self.scope_stack.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                return Some(var_info);
            }
        }
        None
    }

    fn to_html(&mut self) -> String{

        let mut scope_stack: Vec<VariableInfo> = Vec::new(); 

        // HTML code conversion
        let tokens = &self.language_tokens;
        let mut token_strings: Vec<String> =
            tokens.iter().map(|(token, _line)| token.clone()).collect();

        let mut html_string: String = " ".to_string();
        while let Some(token) = token_strings.pop() {
            if token.to_lowercase() == "#hai" {
                html_string.push_str("<!DOCTYPE html> \n<html>");
                continue;

                
            }

            if token.to_lowercase() == "#i" {
                if let Some(token) = token_strings.pop()  {
                    if token.to_lowercase() == "haz"
                    {
                        if let Some(next_token) = token_strings.pop()
                        {
                            let var_name = next_token; 

                            if let Some(next_token) = token_strings.pop()
                            {
                                if next_token.to_lowercase() == "#it"
                                {
                                    if let Some(next_token_iz) = token_strings.pop()
                                    {
                                        if next_token_iz.to_lowercase() == "iz"
                                        {
                                            if let Some(value_token) = token_strings.pop()
                                            {
                                                let var_value = value_token; 

                                                if let Some(mkay_value) = token_strings.pop()
                                                {
                                                    if mkay_value.to_lowercase() == "#mkay"
                                                    {
                                                        //Declare stack
                                                        let variable_info = VariableInfo {
                                                            name: var_name,
                                                            value: Some(var_value),
                                                            line_defined: 0,
                                                        };

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

            if token.to_lowercase() == "#kthxbye" {
                html_string.push_str("\n</html>");
                break;
            }
            if token.to_lowercase() == "#obtw" {
                html_string.push_str("\n<!--");
                while let Some(comment_token) = token_strings.pop() {
                    if comment_token.to_lowercase() == "#tldr" {
                        html_string.push_str(" -->\n");
                        break;
                    }
                    html_string.push_str(" ");
                    html_string.push_str(&comment_token);
                }
            }

            if token.to_lowercase() == "#maek" {
                if let Some(next_token) = token_strings.pop() {
                    if next_token.to_lowercase() == "head" {
                        html_string.push_str("\n<head>");

                        while let Some(head_token) = token_strings.pop() {
                            if head_token.to_lowercase() == "#oic" {
                                html_string.push_str("</head>\n");
                                break;
                            }

                            if head_token.to_lowercase() == "#gimmeh" {
                                if let Some(title_token) = token_strings.pop() {
                                    if title_token.to_lowercase() == "title" {
                                        html_string.push_str("\n<title>");
                                        while let Some(text_token) = token_strings.pop() {
                                            if text_token.to_lowercase() == "#mkay" {
                                                html_string.push_str("</title>\n");
                                                break;
                                            }
                                            html_string.push_str(" ");
                                            html_string.push_str(&text_token);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if next_token.to_lowercase() == "paragraf" {
                        html_string.push_str("\n<p>");
                        while let Some(para_token) = token_strings.pop() {
                            if para_token.to_lowercase() == "#oic" {
                                html_string.push_str("</p>\n");
                            if scope_stack.len() > 1
                            {
                                scope_stack.pop();
                            }
                                break;
                            }

                             if para_token.to_lowercase() == "#i" {
                if let Some(token) = token_strings.pop()  {
                    if token.to_lowercase() == "haz"
                    {
                        if let Some(next_token) = token_strings.pop()
                        {
                            let var_name = next_token; 

                            if let Some(next_token) = token_strings.pop()
                            {
                                if next_token.to_lowercase() == "#it"
                                {
                                    if let Some(next_token_iz) = token_strings.pop()
                                    {
                                        if next_token_iz.to_lowercase() == "iz"
                                        {
                                            if let Some(value_token) = token_strings.pop()
                                            {
                                                let var_value = value_token; 

                                                if let Some(mkay_value) = token_strings.pop()
                                                {
                                                    if mkay_value.to_lowercase() == "#mkay"
                                                    {
                                                        //Declare stack
                                                        let variable_info = VariableInfo {
                                                            name: var_name,
                                                            value: Some(var_value),
                                                            line_defined: 0,
                                                        };

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

                if para_token.to_lowercase() == "#lemme"
                                            {
                                                if let Some(variable_lemme) = token_strings.pop()
                                                {
                                                    if variable_lemme.to_lowercase() == "see"
                                                    {
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
                            if para_token.to_lowercase() == "#maek" {
                                if let Some(list_token) = token_strings.pop() {
                                    if list_token.to_lowercase() == "list" {
                                        html_string.push_str("\n<ul>");
                                        while let Some(list_elem_token) = token_strings.pop() {
                                            if list_elem_token.to_lowercase() == "#oic" {
                                                html_string.push_str("\n</ul>\n");
                                                break;
                                            }

                                            if list_elem_token.to_lowercase() == "#gimmeh" {
                                                if let Some(item_token) = token_strings.pop() {
                                                    if item_token.to_lowercase() == "item" {
                                                        html_string.push_str("\n<li>");
                                                        while let Some(item_content_token) =
                                                                token_strings.pop()
                                                                
                                                        {

                                                            if item_content_token.to_lowercase() == "#lemme"
                                            {
                                                if let Some(variable_lemme) = token_strings.pop()
                                                {
                                                    if variable_lemme.to_lowercase() == "see"
                                                    {
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

                            if para_token.to_lowercase() == "#gimmeh" {
                                if let Some(para_elem_token) = token_strings.pop() {
                                    if para_elem_token.to_lowercase() == "newline" {
                                        html_string.push_str("\n<br/>\n");
                                    }

                                    if para_elem_token.to_lowercase() == "soundz" {
                                        if let Some(address_token) = token_strings.pop() {
                                            html_string.push_str(&format!(
                                                "\n<audio controls>\n<source src=\"{}\" type=\"audio/mpeg\">",
                                                address_token
                                            ));
                                        }

                                        if let  Some(address_token) = token_strings.pop() {
                                            if address_token.to_lowercase() == "#mkay" {
                                                html_string.push_str("</audio>\n");
                                            }
                            
                                        } 
                                    }
    
                                    
                                    if para_elem_token.to_lowercase() == "vidz" 
                                    {
                                        if let Some(address_token) = token_strings.pop() {
                                            html_string.push_str(&format!(
                                                "\n<iframe src = {}>\n",
                                                address_token
                                            ));
                                        }

                                        if let  Some(address_token) = token_strings.pop() {
                                            if address_token.to_lowercase() == "#mkay" {
                                                continue;
                                            }
                            
                                        }
                                    }
                                    if para_elem_token.to_lowercase() == "bold" {
                                        html_string.push_str(" <b>");
                                        while let Some(bold_token) = token_strings.pop() {
                                            if bold_token.to_lowercase() == "#lemme"
                                            {
                                                if let Some(variable_lemme) = token_strings.pop()
                                                {
                                                    if variable_lemme.to_lowercase() == "see"
                                                    {
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

                                            if bold_token.to_lowercase() == "#mkay" {
                                                html_string.push_str(" </b>");
                                                break;
                                            }
                                            html_string.push_str(" ");
                                            html_string.push_str(&bold_token);
                                        }
                                    }

                                    if para_elem_token.to_lowercase() == "italics" {
                                        html_string.push_str(" <i>");
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

                            } else if !para_token.starts_with("#") {
                                html_string.push_str(" ");
                                html_string.push_str(&para_token);
                            }

                        
                    }

                }

                       
                }

                    
            }

            else  {
                if token.to_lowercase() == "#lemme"
                                            {
                                                if let Some(variable_lemme) = token_strings.pop()
                                                {
                                                    if variable_lemme.to_lowercase() == "see"
                                                    {
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
                else if !token.starts_with("#")
                {
                    html_string.push_str(" ");
                    html_string.push_str(&token);                }
                else  {
                   continue;
                }

            }
        }

             html_string

    }

}


impl Compiler for LolcodeCompiler {
    fn compile(&mut self, source: &str) {
        self.lexer = LolcodeLexicalAnalyzer::new(source);
        self.lexer.tokenize();
        self.language_tokens = self.lexer.tokens.clone();
        self.start();
    }

    fn next_token(&mut self) -> String {
        let result = self.lexer.tokens.pop();

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
        } else {
            self.current_tok.clear();
            String::new()
        }
    }

    fn parse(&mut self) {
        self.lolcode();

        if !self.lexer.tokens.is_empty() {
            eprintln!(
                "Syntax error at line {}: Additional tokens found after the document.",
                self.parser.current_line
            );
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

pub fn open_html_in_chrome<P: AsRef<Path>>(html_file: P) -> io::Result<()> {
    let p = html_file.as_ref();
    
    if !p.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", p.display()),
        ));
    }
    
    let abs = fs::canonicalize(p)?;
    
    // Handle potential non-UTF8 paths gracefully
    let path_str = abs.to_str().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Path contains invalid UTF-8")
    })?;
    
    let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);
    
    // Convert to file:// URL
    let file_url = format!("file:///{}", clean_path.replace('\\', "/"));
    
    println!("Opening in Chrome: {}", file_url);
    
    let status = Command::new("cmd")
        .args(&["/C", "start", "chrome", &file_url])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Chrome exited with status: {}", status),
        ))
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
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
    Some(other) => {
        println!("Error: Invalid file extension '.{}'. Only .lol files are accepted.", other);
        process::exit(1);
    }
    None => {
        println!("Error: No file extension found. Only .lol files are accepted.");
        process::exit(1);
    }
}

let html_filename = file_path
.file_stem()
.and_then(|name| name.to_str())
.map(|name| format!("{}.html", name))
.unwrap_or_else(|| "output.html".to_string()); 

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


    let html_string: String = compiler.to_html();


    std::fs::write(&html_filename, html_string).expect("Unable to write file"); 

    open_html_in_chrome(&html_filename); 
    
    println!("This lolcode script is syntactically valid.");
    println!("Static semantic analysis passed: All variables are properly defined before use.");
}