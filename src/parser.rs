use crate::ast;
use crate::lexer::{Token, TokenKind};

use std::process::exit;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token { 
        &self.tokens[self.pos] 
    }

    fn peek_n(&self, n: usize) -> &Token {
        return &self.tokens[self.pos + n];
    }

    fn check_kind(&self, kind: TokenKind) -> bool { 
        self.peek().kind == kind 
    }

    fn advance(&mut self) -> Token { 
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        return tok;
    }

    fn match_(&mut self, kind: TokenKind) -> bool {
        if self.check_kind(kind) { true } 
        else { false }
    }

    fn except(&mut self, token: Token) -> bool {
        if self.match_(token.kind) {
            return true;
        }
        println!("Parser error: unexpected token '{:?}'", token.content);
        exit(-1);
    }

    fn parse_expression(&mut self) -> ast::Expr {
        let x = self.peek();
        match x.kind {
            TokenKind::This => {
                // self.parse_this();
            }
            TokenKind::Identifier => {
                // self.parse_identifier();
            }
            TokenKind::True 
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Number
                | TokenKind::String
                => {
                // self.parse_literal();
            }
            TokenKind::OpenBracket => {
                // self.parse_array();
            }
            TokenKind::OpenCurly => {
                // self.parse_object();
            }
            TokenKind::OpenParen => { // ( Expression )
                self.advance();
                // self.parse_expression();
            }
            _ => {}
        }
        return ast::Expr::This;
    }
}
