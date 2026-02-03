use std::process::exit;

pub struct Cursor {
    pub line: usize,
    pub row: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TokenKind {
    // keywords
    Break,
    For,
    New,
    Var,
    Continue,
    Function,
    Return,
    Void,
    Delete,
    If,
    This,
    While,
    Else,
    In,
    Typeof,
    With,
    True,
    False,
    Null,

    // future reserved keywords
    Case,
    Debugger,
    Export,
    Super,
    Catch,
    Default, // even if Default is a Rust keyword, it does not generate a warning or error
    Extends,
    Switch,
    Class,
    Do,
    Finally,
    Throw,
    Const,
    Enum,
    Import,
    Try,
    Undefined,

    // symbols
    SemiColon,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Dot,
    Slash,
    Asterisk,
    Equal,
    GreaterThan,
    LessThan,
    DoubleEqual,
    LessThanEqual,
    GreaterThanEqual,
    NotEqual,
    Comma,
    Exclamation,
    Wave, // ~
    Question,
    DoubleDot,
    And, // &&
    Or,  /* || */
    DoublePlus,
    DoubleMinus,
    Plus,
    Minus,
    Ampersand,
    Bar,   // |
    Caret, /* ^ */
    Modulo,
    LeftShift,
    RightShift,
    TripleGreaterThan,
    PlusEqual,
    MinusEqual,
    AsteriskEqual,
    SlashEqual,
    AmpersandEqual,
    BarEqual,
    CaretEqual,
    ModuloEqual,
    LeftShiftEqual,
    RightShiftEqual,
    TripleGreaterThanEqual,
    OpenCurly,
    CloseCurly,
    BackSlash,

    Identifier,
    Number,
    String,
    NewLine,
    EOF,
}

#[derive(Clone)]
pub struct Token {
    pub content: String,
    pub kind: TokenKind,
    pub line_terminator_before: bool,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    pub source: String,
    pub cursor: Cursor,
    pub line: usize,
    pub row: usize,
    pub prev_cr: bool,
}

impl Lexer {
    fn get_next_char(&mut self) -> char {
        let c = self
            .source
            .chars()
            .nth({
                let tmp = self.row;
                self.row += 1;
                tmp
            })
            .unwrap_or('\0');

        if c == '\0' {
            return c;
        }

        if c == '\r' {
            self.cursor.line += 1;
            self.cursor.row = 0;
            self.prev_cr = true;
        } else if c == '\n' {
            if self.prev_cr {
                self.prev_cr = false;
            } else {
                self.cursor.line += 1;
                self.cursor.row = 0;
            }
        } else {
            self.cursor.row += 1;
            self.prev_cr = false;
        }

        return c;
    }

    fn keyword_kind(s: &str) -> TokenKind {
        match s {
            "break" => TokenKind::Break,
            "for" => TokenKind::For,
            "new" => TokenKind::New,
            "var" => TokenKind::Var,
            "continue" => TokenKind::Continue,
            "function" => TokenKind::Function,
            "return" => TokenKind::Return,
            "void" => TokenKind::Void,
            "delete" => TokenKind::Delete,
            "if" => TokenKind::If,
            "this" => TokenKind::This,
            "while" => TokenKind::While,
            "else" => TokenKind::Else,
            "in" => TokenKind::In,
            "typeof" => TokenKind::Typeof,
            "with" => TokenKind::With,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "undefined" => TokenKind::Undefined,

            "case" => TokenKind::Case,
            "debugger" => TokenKind::Debugger,
            "export" => TokenKind::Export,
            "super" => TokenKind::Super,
            "catch" => TokenKind::Catch,
            "default" => TokenKind::Default,
            "extends" => TokenKind::Extends,
            "switch" => TokenKind::Switch,
            "class" => TokenKind::Class,
            "do" => TokenKind::Do,
            "finally" => TokenKind::Finally,
            "throw" => TokenKind::Throw,
            "const" => TokenKind::Const,
            "enum" => TokenKind::Enum,
            "import" => TokenKind::Import,
            "try" => TokenKind::Try,

            _ => TokenKind::Identifier,
        }
    }

    fn get_current_char(&self) -> char {
        return self.source.chars().nth(self.row).unwrap_or('\0');
    }

    fn peek_char(&self, offset: usize) -> char {
        return self.source.chars().nth(self.row + offset).unwrap_or('\0');
    }

    fn eat_char(&mut self, expected: char) -> bool {
        if self.get_current_char() == expected {
            self.get_next_char();
            return true;
        }
        return false;
    }

    fn isspace(x: char) -> bool {
        match x {
            '\t' | '\u{000B}' | '\u{000C}' | ' ' => return true,
            _ => return false,
        }
    }

    fn isterminator(x: char) -> bool {
        match x {
            '\u{000D}' | '\u{000A}' => return true,
            _ => return false,
        }
    }

    fn error(&self, msg: &str) -> ! {
        println!(
            "Lexer error at {}:{}: {}",
            self.cursor.line + 1,
            self.cursor.row + 1,
            msg
        );
        exit(-1);
    }

    fn skip_comment(&mut self) -> bool {
        if self.get_current_char() != '/' {
            return false;
        }

        match self.peek_char(1) {
            '/' => {
                self.get_next_char();
                self.get_next_char();

                while {
                    let c = self.get_current_char();
                    c != '\0' && !Self::isterminator(c)
                } {
                    self.get_next_char();
                }

                true
            }

            '*' => {
                self.get_next_char();
                self.get_next_char();

                let mut prev = '\0';
                loop {
                    let c = self.get_next_char();

                    if c == '\0' {
                        self.error("EOF in a comment");
                    }

                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }

                true
            }

            _ => false,
        }
    }

    fn skip_spaces(&mut self) {
        loop {
            while Self::isspace(self.get_current_char()) {
                self.get_next_char();
            }
            if self.skip_comment() {
                continue;
            }
            break;
        }
    }

    pub fn next(&mut self) -> Token {
        let mut saw_line_terminator = false;
        loop {
            self.skip_spaces();
            let c = self.get_current_char();
            if c == '\u{000D}' {
                self.get_next_char();
                self.eat_char('\u{000A}');
                saw_line_terminator = true;
                continue;
            }
            if c == '\u{000A}' {
                self.get_next_char();
                saw_line_terminator = true;
                continue;
            }
            break;
        }

        let start_line = self.cursor.line;
        let start_col = self.cursor.row;

        let mut token = Token {
            kind: TokenKind::EOF,
            content: "EOF".to_string(),
            line_terminator_before: saw_line_terminator,
            line: start_line,
            col: start_col,
        };

        let x: char = self.get_next_char();
        if x == '\0' {
            return token;
        }

        match x {
            '(' => {
                token.content = "(".to_string();
                token.kind = TokenKind::OpenParen;
                return token;
            }
            ')' => {
                token.content = ")".to_string();
                token.kind = TokenKind::CloseParen;
                return token;
            }
            '{' => {
                token.content = "{".to_string();
                token.kind = TokenKind::OpenCurly;
                return token;
            }
            '}' => {
                token.content = "}".to_string();
                token.kind = TokenKind::CloseCurly;
                return token;
            }
            '[' => {
                token.content = "[".to_string();
                token.kind = TokenKind::OpenBracket;
                return token;
            }
            ']' => {
                token.content = "]".to_string();
                token.kind = TokenKind::CloseBracket;
                return token;
            }
            ';' => {
                token.content = ";".to_string();
                token.kind = TokenKind::SemiColon;
                return token;
            }
            '.' => {
                token.content = ".".to_string();
                token.kind = TokenKind::Dot;
                return token;
            }
            ':' => {
                token.content = ":".to_string();
                token.kind = TokenKind::DoubleDot;
                return token;
            }
            '\\' => {
                token.content = "\\".to_string();
                token.kind = TokenKind::BackSlash;
                return token;
            }
            '*' => {
                if self.eat_char('=') {
                    token.content = "*=".to_string();
                    token.kind = TokenKind::AsteriskEqual;
                } else {
                    token.content = "*".to_string();
                    token.kind = TokenKind::Asterisk;
                }
                return token;
            }
            '/' => {
                if self.eat_char('=') {
                    token.content = "/=".to_string();
                    token.kind = TokenKind::SlashEqual;
                } else {
                    token.content = "/".to_string();
                    token.kind = TokenKind::Slash;
                }
                return token;
            }
            '&' => {
                if self.eat_char('&') {
                    token.content = "&&".to_string();
                    token.kind = TokenKind::And;
                } else if self.eat_char('=') {
                    token.content = "&=".to_string();
                    token.kind = TokenKind::AmpersandEqual;
                } else {
                    token.content = "&".to_string();
                    token.kind = TokenKind::Ampersand;
                }
                return token;
            }
            '|' => {
                if self.eat_char('|') {
                    token.content = "||".to_string();
                    token.kind = TokenKind::Or;
                } else if self.eat_char('=') {
                    token.content = "|=".to_string();
                    token.kind = TokenKind::BarEqual;
                } else {
                    token.content = "|".to_string();
                    token.kind = TokenKind::Bar;
                }
                return token;
            }
            '^' => {
                if self.eat_char('=') {
                    token.content = "^=".to_string();
                    token.kind = TokenKind::CaretEqual;
                } else {
                    token.content = "^".to_string();
                    token.kind = TokenKind::Caret;
                }
                return token;
            }
            '%' => {
                if self.eat_char('=') {
                    token.content = "%=".to_string();
                    token.kind = TokenKind::ModuloEqual;
                } else {
                    token.content = "%".to_string();
                    token.kind = TokenKind::Modulo;
                }
                return token;
            }
            '~' => {
                token.content = "~".to_string();
                token.kind = TokenKind::Wave;
                return token;
            }
            '?' => {
                token.content = "?".to_string();
                token.kind = TokenKind::Question;
                return token;
            }
            '=' => {
                if self.eat_char('=') {
                    token.content = "==".to_string();
                    token.kind = TokenKind::DoubleEqual;
                } else {
                    token.content = "=".to_string();
                    token.kind = TokenKind::Equal;
                }
                return token;
            }
            '<' => {
                if self.eat_char('=') {
                    token.content = "<=".to_string();
                    token.kind = TokenKind::LessThanEqual;
                } else if self.eat_char('<') {
                    if self.eat_char('=') {
                        token.content = "<<=".to_string();
                        token.kind = TokenKind::LeftShiftEqual;
                    } else {
                        token.content = "<<".to_string();
                        token.kind = TokenKind::LeftShift;
                    }
                } else {
                    token.content = "<".to_string();
                    token.kind = TokenKind::LessThan;
                }
                return token;
            }
            '\'' | '"' => {
                let delimiter = x;
                let mut s = String::new();
                s.push(x);

                loop {
                    let c = self.get_next_char();
                    if c == '\0' {
                        self.error("EOF in string");
                    }
                    if c == '\\' {
                        s.push(c);
                        let next = self.get_next_char();
                        if next == '\0' {
                            self.error("EOF in string escape");
                        }
                        s.push(next);
                        continue;
                    }
                    s.push(c);
                    if c == delimiter {
                        break;
                    }
                }

                token.content = s;
                token.kind = TokenKind::String;
                return token;
            }
            '>' => {
                if self.eat_char('=') {
                    token.content = ">=".to_string();
                    token.kind = TokenKind::GreaterThanEqual;
                } else if self.eat_char('>') {
                    if self.eat_char('>') {
                        if self.eat_char('=') {
                            token.content = ">>>=".to_string();
                            token.kind = TokenKind::TripleGreaterThanEqual;
                        } else {
                            token.content = ">>>".to_string();
                            token.kind = TokenKind::TripleGreaterThan;
                        }
                    } else if self.eat_char('=') {
                        token.content = ">>=".to_string();
                        token.kind = TokenKind::RightShiftEqual;
                    } else {
                        token.content = ">>".to_string();
                        token.kind = TokenKind::RightShift;
                    }
                } else {
                    token.content = ">".to_string();
                    token.kind = TokenKind::GreaterThan;
                }
                return token;
            }
            '!' => {
                if self.eat_char('=') {
                    token.content = "!=".to_string();
                    token.kind = TokenKind::NotEqual;
                } else {
                    token.content = "!".to_string();
                    token.kind = TokenKind::Exclamation;
                }
                return token;
            }
            '+' => {
                if self.eat_char('+') {
                    token.content = "++".to_string();
                    token.kind = TokenKind::DoublePlus;
                } else if self.eat_char('=') {
                    token.content = "+=".to_string();
                    token.kind = TokenKind::PlusEqual;
                } else {
                    token.content = "+".to_string();
                    token.kind = TokenKind::Plus;
                }
                return token;
            }
            '-' => {
                if self.eat_char('-') {
                    token.content = "--".to_string();
                    token.kind = TokenKind::DoubleMinus;
                } else if self.eat_char('=') {
                    token.content = "-=".to_string();
                    token.kind = TokenKind::MinusEqual;
                } else {
                    token.content = "-".to_string();
                    token.kind = TokenKind::Minus;
                }
                return token;
            }
            _ => {
                if x.is_numeric() {
                    let mut s = String::new();
                    s.push(x);

                    while {
                        let c = self.get_current_char();
                        c != '\0' && (c.is_numeric() || c == '_' || c == '.' || c == 'x')
                    } {
                        s.push(self.get_next_char());
                    }

                    if self.get_current_char() == 'e' || self.get_current_char() == 'E' {
                        s.push(self.get_next_char());
                        if self.get_current_char() == '+' || self.get_current_char() == '-' {
                            s.push(self.get_next_char());
                        }
                        while {
                            let c = self.get_current_char();
                            c != '\0' && c.is_numeric()
                        } {
                            s.push(self.get_next_char());
                        }
                    }

                    let next = self.get_current_char();
                    if next.is_alphabetic() || next == '$' || next == '_' {
                        self.error("missing separator after number literal");
                    }

                    token.content = s;
                    token.kind = TokenKind::Number;
                    return token;
                }
                if x.is_alphabetic() || x == '$' || x == '_' {
                    let mut s = String::new();
                    s.push(x);

                    while {
                        let c = self.get_current_char();
                        c != '\0' && (c.is_alphanumeric() || c == '_')
                    } {
                        s.push(self.get_next_char());
                    }

                    token.content = s;
                    token.kind = Self::keyword_kind(&token.content);

                    return token;
                }
                self.error(&format!("Unknown token start '{}'", x));
            }
        }
    }

    pub fn walk(&mut self) -> Vec<Token> {
        let mut output: Vec<Token> = vec![];
        loop {
            let token = self.next();
            output.push(token.clone());
            if token.kind == TokenKind::EOF {
                break;
            }
        }
        return output;
    }
}
