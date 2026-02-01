#![allow(dead_code)]

// You should read this with the ECMAScript Third Edition on Annex B

use crate::ast;
use crate::lexer::{Token, TokenKind};

use std::process::exit;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub allow_in: bool,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_n(&self, n: usize) -> &Token {
        let idx = self.pos + n;
        if idx >= self.tokens.len() {
            return &self.tokens[self.tokens.len() - 1];
        }
        &self.tokens[idx]
    }

    fn check_kind(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        return tok;
    }

    fn match_(&mut self, kind: TokenKind) -> bool {
        if self.check_kind(kind) {
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: Token) -> bool {
        if self.match_(token.kind) {
            return true;
        }
        println!("Parser error: unexpected token '{:?}'", token.content);
        exit(-1);
    }

    fn parse_expression(&mut self) -> ast::Expr {
        let mut elements: Vec<ast::Expr> = vec![];

        loop {
            elements.push(self.parse_assignment_expression());

            if self.peek().kind != TokenKind::Comma {
                break;
            }
            self.advance();
        }

        if elements.len() == 1 {
            elements.remove(0)
        } else {
            ast::Expr::Sequence(elements)
        }
    }

    fn parse_primary_expression(&mut self) -> ast::Expr {
        let x = self.peek();
        match x.kind {
            TokenKind::This => {
                self.advance();
                return ast::Expr::This;
            }
            TokenKind::Identifier => {
                let name = x.content.clone();
                self.advance();
                return ast::Expr::Identifier(name);
            }
            TokenKind::String => {
                return ast::Expr::Literal(ast::Literal::String(x.content.clone()));
            }
            TokenKind::True => {
                return ast::Expr::Literal(ast::Literal::Bool(true));
            }
            TokenKind::False => {
                return ast::Expr::Literal(ast::Literal::Bool(false));
            }
            TokenKind::Null => {
                return ast::Expr::Literal(ast::Literal::Null);
            }
            TokenKind::Number => {
                let x_content = x.content.clone();
                self.advance();
                return ast::Expr::Literal(ast::Literal::Number(x_content.parse().unwrap()));
            }
            TokenKind::OpenBracket => {
                self.advance();
                return self.parse_array();
            }
            TokenKind::OpenCurly => {
                // self.parse_object();
            }
            TokenKind::OpenParen => {
                // ( Expression )
                self.advance();
                let expr = self.parse_expression();

                if self.peek().kind != TokenKind::CloseParen {
                    println!(
                        "Parser error: Unexpected token '{}', expected ')'",
                        self.peek().content
                    );
                    exit(-1);
                }

                self.advance();
                return expr;
            }
            _ => {
                println!("Parser error: unexpected token '{}' in expression", x.content);
                exit(-1);
            }
        }
        return ast::Expr::This;
    }

    fn parse_array(&mut self) -> ast::Expr {
        let mut elements: Vec<ast::Expr> = vec![];

        if self.peek().kind == TokenKind::CloseBracket {
            self.advance();
            return ast::Expr::Literal(ast::Literal::Array(elements));
        }

        loop {
            if self.peek().kind == TokenKind::Comma {
                self.advance();
                elements.push(ast::Expr::Literal(ast::Literal::Undefined));
                continue;
            }

            elements.push(self.parse_assignment_expression());

            match self.peek().kind {
                TokenKind::Comma => {
                    self.advance();
                    if self.peek().kind == TokenKind::CloseBracket {
                        elements.push(ast::Expr::Literal(ast::Literal::Undefined));
                    }
                }
                TokenKind::CloseBracket => {
                    self.advance();
                    break;
                }
                _ => {
                    println!("Parser error: expected ',' or ']' in array");
                    exit(-1);
                }
            }
        }

        ast::Expr::Literal(ast::Literal::Array(elements))
    }

    fn parse_assignment_expression(&mut self) -> ast::Expr {
        let left = self.parse_conditional_expression();

        let kind = &self.peek().kind;
        if *kind == TokenKind::Equal
            || *kind == TokenKind::PlusEqual
            || *kind == TokenKind::MinusEqual
            || *kind == TokenKind::AsteriskEqual
            || *kind == TokenKind::SlashEqual
            || *kind == TokenKind::ModuloEqual
            || *kind == TokenKind::LeftShiftEqual
            || *kind == TokenKind::RightShiftEqual
            || *kind == TokenKind::TripleGreaterThanEqual
            || *kind == TokenKind::AmpersandEqual
            || *kind == TokenKind::CaretEqual
            || *kind == TokenKind::BarEqual
        {
            let assignement_op = self.parse_assignment_operator();
            let expr = self.parse_assignment_expression();

            return ast::Expr::Assign {
                target: Box::new(left),
                op: assignement_op,
                value: Box::new(expr),
            };
        }

        return left;
    }

    fn parse_assignment_operator(&mut self) -> ast::AssignOp {
        let x = self.advance();

        match x.kind {
            TokenKind::Equal => {
                return ast::AssignOp::Assign;
            }
            TokenKind::PlusEqual => {
                return ast::AssignOp::AddAssign;
            }
            TokenKind::MinusEqual => {
                return ast::AssignOp::SubAssign;
            }
            TokenKind::AsteriskEqual => {
                return ast::AssignOp::MulAssign;
            }
            TokenKind::SlashEqual => {
                return ast::AssignOp::DivAssign;
            }
            TokenKind::ModuloEqual => {
                return ast::AssignOp::ModAssign;
            }
            TokenKind::LeftShiftEqual => {
                return ast::AssignOp::ShlAssign;
            }
            TokenKind::RightShiftEqual => {
                return ast::AssignOp::ShrAssign;
            }
            TokenKind::TripleGreaterThanEqual => {
                return ast::AssignOp::UShrAssign;
            }
            TokenKind::AmpersandEqual => {
                return ast::AssignOp::BitAndAssign;
            }
            TokenKind::CaretEqual => {
                return ast::AssignOp::BitXorAssign;
            }
            TokenKind::BarEqual => {
                return ast::AssignOp::BitOrAssign;
            }
            _ => {
                println!("Parser error: illegal assignement operator '{}'", x.content);
                exit(-1);
            }
        }
    }

    fn parse_conditional_expression(&mut self) -> ast::Expr {
        let logic_or_expr = self.parse_logical_or_expression();

        if self.check_kind(TokenKind::Question) {
            let assign_expr = self.parse_assignment_expression();
            let assign_expr2;

            if self.check_kind(TokenKind::DoubleDot) {
                assign_expr2 = self.parse_assignment_expression();

                return ast::Expr::Ternary{
                    cond: Box::new(logic_or_expr),
                    then_: Box::new(assign_expr),
                    else_: Box::new(assign_expr2),
                };
            } else {
                println!("Parser error: expected ':' in conditional expression but found '{}'", self.peek().content);
                exit(-1);
            }
        }

        return logic_or_expr;
    }

    fn parse_logical_or_expression(&mut self) -> ast::Expr {
        let logic_and_expr = self.parse_logical_and_expression();

        if self.peek().kind == TokenKind::Or {
            self.advance();
            let logic_and_expr2 = self.parse_logical_and_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Or,
                left: Box::new(logic_and_expr),
                right: Box::new(logic_and_expr2),
            };
        }

        return logic_and_expr;
    }

    fn parse_logical_and_expression(&mut self) -> ast::Expr {
        let bitwise_or_expr = self.parse_bitwise_or_expression();

        if self.peek().kind == TokenKind::And {
            self.advance();
            let bitwise_or_expr2 = self.parse_bitwise_or_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::And,
                left: Box::new(bitwise_or_expr),
                right: Box::new(bitwise_or_expr2),
            };
        }

        return bitwise_or_expr;
    }

    fn parse_bitwise_or_expression(&mut self) -> ast::Expr {
        let bitwise_xor_expr = self.parse_bitwise_xor_expression();

        if self.peek().kind == TokenKind::Bar {
            self.advance();
            let bitwise_xor_expr2 = self.parse_bitwise_xor_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::BitOr,
                left: Box::new(bitwise_xor_expr),
                right: Box::new(bitwise_xor_expr2),
            };
        }

        return bitwise_xor_expr;
    }

    fn parse_bitwise_xor_expression(&mut self) -> ast::Expr {
        let bitwise_and_expr = self.parse_bitwise_and_expression();

        if self.peek().kind == TokenKind::Caret {
            self.advance();
            let bitwise_and_expr2 = self.parse_bitwise_and_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::BitXor,
                left: Box::new(bitwise_and_expr),
                right: Box::new(bitwise_and_expr2),
            };
        }

        return bitwise_and_expr;
    }

    fn parse_bitwise_and_expression(&mut self) -> ast::Expr {
        let eq_expr = self.parse_equality_expression();

        if self.peek().kind == TokenKind::Ampersand {
            self.advance();
            let eq_expr2 = self.parse_equality_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::BitAnd,
                left: Box::new(eq_expr),
                right: Box::new(eq_expr2),
            };
        }

        return eq_expr;
    }

    fn parse_equality_expression(&mut self) -> ast::Expr {
        let relational_expr = self.parse_relational_expression();

        if self.peek().kind == TokenKind::DoubleEqual {
            self.advance();
            let relational_expr2 = self.parse_relational_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Eq,
                left: Box::new(relational_expr),
                right: Box::new(relational_expr2),
            };
        } else if self.peek().kind == TokenKind::NotEqual {
            self.advance();
            let relational_expr2 = self.parse_relational_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Ne,
                left: Box::new(relational_expr),
                right: Box::new(relational_expr2),
            };
        }

        return relational_expr;
    }

    // TODO: Add 'in' support
    // A problem is that we do not have a 'In' node in the AST,
    // only a 'ForIn' in ast::Stmt ...
    fn parse_relational_expression(&mut self) -> ast::Expr {
        let shift_expr = self.parse_shift_expression();

        if self.peek().kind == TokenKind::LessThan {
            self.advance();
            let shift_expr2 = self.parse_shift_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Lt,
                left: Box::new(shift_expr),
                right: Box::new(shift_expr2),
            };
        } else if self.peek().kind == TokenKind::GreaterThan {
            self.advance();
            let shift_expr2 = self.parse_shift_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Gt,
                left: Box::new(shift_expr),
                right: Box::new(shift_expr2),
            };
        } else if self.peek().kind == TokenKind::GreaterThanEqual {
            self.advance();
            let shift_expr2 = self.parse_shift_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Ge,
                left: Box::new(shift_expr),
                right: Box::new(shift_expr2),
            };
        } else if self.peek().kind == TokenKind::LessThanEqual {
            self.advance();
            let shift_expr2 = self.parse_shift_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Le,
                left: Box::new(shift_expr),
                right: Box::new(shift_expr2),
            };
        } else if self.peek().kind == TokenKind::In && self.allow_in {
            self.advance();
            let shift_expr2 = self.parse_shift_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::In,
                left: Box::new(shift_expr),
                right: Box::new(shift_expr2),
            };
        }

        return shift_expr;
    }

    fn parse_shift_expression(&mut self) -> ast::Expr {
        let add_expr = self.parse_additive_expression();

        if self.peek().kind == TokenKind::LeftShift {
            self.advance();
            let add_expr2 = self.parse_additive_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Shl,
                left: Box::new(add_expr),
                right: Box::new(add_expr2),
            };
        } else if self.peek().kind == TokenKind::RightShift {
            self.advance();
            let add_expr2 = self.parse_additive_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Shr,
                left: Box::new(add_expr),
                right: Box::new(add_expr2),
            };
        } else if self.peek().kind == TokenKind::TripleGreaterThan {
            self.advance();
            let add_expr2 = self.parse_additive_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::UShr,
                left: Box::new(add_expr),
                right: Box::new(add_expr2),
            };
        }

        return add_expr;
    }

    fn parse_additive_expression(&mut self) -> ast::Expr {
        let mul_expr = self.parse_multiplicative_expression();

        if self.peek().kind == TokenKind::Plus {
            self.advance();
            let mul_expr2 = self.parse_multiplicative_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Add,
                left: Box::new(mul_expr),
                right: Box::new(mul_expr2),
            };
        } else if self.peek().kind == TokenKind::Minus {
            self.advance();
            let mul_expr2 = self.parse_multiplicative_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Sub,
                left: Box::new(mul_expr),
                right: Box::new(mul_expr2),
            };
        }

        return mul_expr;
    }

    fn parse_multiplicative_expression(&mut self) -> ast::Expr {
        let un_expr = self.parse_unary_expression();

        if self.peek().kind == TokenKind::Asterisk {
            self.advance();
            let un_expr2 = self.parse_unary_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Mul,
                left: Box::new(un_expr),
                right: Box::new(un_expr2),
            };
        } else if self.peek().kind == TokenKind::Slash {
            self.advance();
            let un_expr2 = self.parse_unary_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Div,
                left: Box::new(un_expr),
                right: Box::new(un_expr2),
            };
        } else if self.peek().kind == TokenKind::Modulo {
            self.advance();
            let un_expr2 = self.parse_unary_expression();

            return ast::Expr::Binary {
                op: ast::BinOp::Mod,
                left: Box::new(un_expr),
                right: Box::new(un_expr2),
            };
        }

        return un_expr;
    }

    fn parse_unary_expression(&mut self) -> ast::Expr {
        let tok = self.peek();

        match tok.kind {
            TokenKind::Delete => {
                self.advance();
                let expr = self.parse_unary_expression();
                ast::Expr::Unary {
                    op: ast::UnaryOp::Delete,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Void => {
                self.advance();
                let expr = self.parse_unary_expression();
                ast::Expr::Unary {
                    op: ast::UnaryOp::Void,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Typeof => {
                self.advance();
                let expr = self.parse_unary_expression();
                ast::Expr::Unary {
                    op: ast::UnaryOp::Typeof,
                    expr: Box::new(expr),
                }
            }
            TokenKind::DoublePlus => {
                self.advance();
                let expr = self.parse_unary_expression();
                ast::Expr::Update {
                    op: ast::UpdateOp::Inc,
                    prefix: true,
                    argument: Box::new(expr),
                }
            }
            TokenKind::DoubleMinus => {
                self.advance();
                let expr = self.parse_unary_expression();
                ast::Expr::Update {
                    op: ast::UpdateOp::Dec,
                    prefix: true,
                    argument: Box::new(expr),
                }
            }
            TokenKind::Plus => {
                self.advance();
                let expr = self.parse_unary_expression();
                ast::Expr::Unary {
                    op: ast::UnaryOp::Pos,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary_expression();
                ast::Expr::Unary {
                    op: ast::UnaryOp::Neg,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Wave => {
                self.advance();
                let expr = self.parse_unary_expression();
                ast::Expr::Unary {
                    op: ast::UnaryOp::BitNot,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Exclamation => {
                self.advance();
                let expr = self.parse_unary_expression();
                ast::Expr::Unary {
                    op: ast::UnaryOp::Not,
                    expr: Box::new(expr),
                }
            }

            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_postfix_expression(&mut self) -> ast::Expr {
        let expr = self.parse_lefthand_side_expression();

        let tok = self.peek();
        match tok.kind {
            TokenKind::DoublePlus => {
                self.advance();
                ast::Expr::Update {
                    op: ast::UpdateOp::Inc,
                    prefix: false,
                    argument: Box::new(expr),
                }
            }
            TokenKind::DoubleMinus => {
                self.advance();
                ast::Expr::Update {
                    op: ast::UpdateOp::Dec,
                    prefix: false,
                    argument: Box::new(expr),
                }
            }
            _ => expr,
        }
    }

    fn parse_lefthand_side_expression(&mut self) -> ast::Expr {
        let expr = self.parse_new_expression();

        if self.peek().kind == TokenKind::OpenParen {
            self.advance();
            let args = self.parse_arguments();
            return ast::Expr::Call {
                callee: Box::new(expr),
                args: Box::new(args),
            };
        }

        return expr;
    }

    fn parse_new_expression(&mut self) -> ast::Expr {
        return self.parse_member_expression();
    }
 
    fn parse_arguments(&mut self) -> ast::Expr {
        let mut args = vec![];

        if self.peek().kind == TokenKind::CloseParen {
            self.advance();
            return ast::Expr::Sequence(args);
        }

        loop {
            args.push(self.parse_assignment_expression());

            match self.peek().kind {
                TokenKind::Comma => {
                    self.advance();
                }
                TokenKind::CloseParen => {
                    self.advance();
                    break;
                }
                _ => {
                    println!("Parser error: expected ',' or ')' in arguments");
                    exit(-1);
                }
            }
        }

        return ast::Expr::Sequence(args);
    }

    fn parse_argument_list(&mut self) -> Vec<ast::Expr> {
        let mut outvec = vec![];
        while !self.check_kind(TokenKind::CloseParen) {
            outvec.push(self.parse_assignment_expression());
        }
        return outvec;
    }

    fn parse_member_expression(&mut self) -> ast::Expr {
        let mut expr: ast::Expr;

        if self.peek().kind == TokenKind::Function {
            self.advance();
            expr = self.parse_function_expression();
        } else if self.peek().kind == TokenKind::New {
            self.advance();
            let callee = self.parse_member_expression();
            let args = if self.peek().kind == TokenKind::OpenParen {
                self.advance();
                self.parse_arguments()
            } else {
                ast::Expr::Sequence(vec![])
            };
            expr = ast::Expr::New {
                callee: Box::new(callee),
                args: Box::new(args),
            };
        } else {
         expr = self.parse_primary_expression();
        }

        // Chaînage des accès membres
        loop {
            match self.peek().kind {
                TokenKind::OpenBracket => {
                    self.advance();
                    let index = self.parse_expression();
                    if !self.check_kind(TokenKind::CloseBracket) {
                        println!("Parser error: expected ']'");
                        exit(-1);
                    }
                    expr = ast::Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                TokenKind::Dot => {
                    self.advance();
                    let name = self.parse_identifier();
                    expr = ast::Expr::Member {
                        object: Box::new(expr),
                        property: name,
                    };
                }
                _ => break,
            }
        }

        return expr;
    }

    fn parse_identifier(&mut self) -> String {
        let name = self.peek().content.clone();
        self.advance();
        return name;
    }

    fn parse_function_expression(&mut self) -> ast::Expr {
        let mut name: Option<String> = None;
        if self.peek().kind == TokenKind::Identifier {
            name = Some(self.parse_identifier());
        }

        if !self.check_kind(TokenKind::OpenParen) {
            println!("Parser error: expected '(' after function name");
            exit(-1);
        }

        let params = self.parse_parameter_list();

        if !self.check_kind(TokenKind::CloseParen) {
            println!("Parser error: Not found ')' after '('");
            exit(-1);
        }

        if !self.check_kind(TokenKind::OpenCurly) {
            println!("Parser error: expected '{{' after ')'");
            exit(-1);
        }

        let body = self.parse_function_body();

        return ast::Expr::Function(ast::Function { name, params, body });
    }

    fn parse_parameter_list(&mut self) -> Vec<String> {
        let mut outvec = vec![];

        if self.peek().kind == TokenKind::CloseParen {
            return outvec;
        }

        loop {
            if self.peek().kind != TokenKind::Identifier {
                println!("Parser error: expected identifier in parameter list, found '{}'", self.peek().content);
                exit(-1);
            }
            outvec.push(self.parse_identifier());

            match self.peek().kind {
                TokenKind::Comma => {
                    self.advance();
                }
                TokenKind::CloseParen => {
                    break;
                }
                _ => {
                    println!("Parser error: expected ',' or ')' in parameter list");
                    exit(-1);
                }
            }
        }

        outvec
    }

    fn parse_function_body(&mut self) -> Vec<ast::Stmt> {
        let mut body = vec![];

        while self.peek().kind != TokenKind::CloseCurly && self.peek().kind != TokenKind::EOF {
            body.push(self.parse_statement());
        }

        if !self.check_kind(TokenKind::CloseCurly) {
            println!("Parser error: expected '}}' in function body");
            exit(-1);
        } 

        body
    }

    fn parse_function_declaration(&mut self) -> ast::Function {
        let name: String = self.parse_identifier();

        if !self.check_kind(TokenKind::OpenParen) {
            println!("Parser error: expected '(' after function name");
            exit(-1);
        }

        let params = self.parse_parameter_list();

        if !self.check_kind(TokenKind::CloseParen) {
            println!("Parser error: Not found ')' after '('");
            exit(-1);
        }

        if !self.check_kind(TokenKind::OpenCurly) {
            println!("Parser error: expected '{{' after ')'");
            exit(-1);
        }

        let body = self.parse_function_body();

        ast::Function {
            name: Some(name),
            params,
            body,
        }
    }

    fn parse_statement(&mut self) -> ast::Stmt {
        let tok = self.peek();
        match tok.kind {
            TokenKind::Function => {
                return ast::Stmt::Function(self.parse_function_declaration());
            }
            TokenKind::OpenCurly => {
                return self.parse_block();
            }
            TokenKind::SemiColon => {
                self.advance();
                return ast::Stmt::Empty;
            }
            TokenKind::Var => {
                return self.parse_variable_statement();
            }
            TokenKind::If => {
                return self.parse_if_statement();
            }
            TokenKind::Do | TokenKind::While | TokenKind::For => {
                return self.parse_iteration_statement();
            }
            TokenKind::Continue => {
                return self.parse_continue_statement();
            }
            TokenKind::Break => {
                return self.parse_break_statement();
            }
            TokenKind::Return => {
                return self.parse_return_statement();
            }
            TokenKind::With => {
                return self.parse_with_statement();
            }
            _ => {
                // Not Function
                let expr = self.parse_expression();
                self.check_kind(TokenKind::SemiColon);
                return ast::Stmt::Expr(expr);
            }
        }
    }

    fn parse_block(&mut self) -> ast::Stmt {
        if !self.check_kind(TokenKind::OpenCurly) {
            println!("Parser error: expected '{{'");
            exit(-1);
        }

        if self.peek().kind == TokenKind::CloseCurly {
            self.advance();
            return ast::Stmt::Block(vec![]);
        }

        let stmts = self.parse_statement_list();

        if !self.check_kind(TokenKind::CloseCurly) {
            println!("Parser error: expected '}}'");
            exit(-1);
        }

        ast::Stmt::Block(stmts)
    }

    fn parse_statement_list(&mut self) -> Vec<ast::Stmt> {
        let mut stmts: Vec<ast::Stmt> = vec![];

        while self.peek().kind != TokenKind::CloseCurly && self.peek().kind != TokenKind::EOF {
            stmts.push(self.parse_statement());
        }

        stmts
    }

    fn parse_variable_statement(&mut self) -> ast::Stmt {
        if self.check_kind(TokenKind::Var) {
            let vars = self.parse_variable_declaration_list();
            self.check_kind(TokenKind::SemiColon);
            return ast::Stmt::Var(vars);
        }
        println!("Parser error: 'var' expected but not found in parse_variable_statement()");
        exit(-1);
    }

    fn parse_variable_declaration_list(&mut self) -> Vec<(String, Option<ast::Expr>)> {
        let mut vars: Vec<(String, Option<ast::Expr>)> = vec![];

        while self.peek().kind == TokenKind::Identifier {
            let name: String = self.peek().content.clone();
            let mut init: ast::Expr = ast::Expr::Literal(ast::Literal::Undefined);
            self.advance();

            if self.check_kind(TokenKind::Equal) {
                init = self.parse_assignment_expression();
            }

            vars.push((name, Some(init)));
        }

        return vars;
    }

    fn parse_if_statement(&mut self) -> ast::Stmt {
        if self.check_kind(TokenKind::If) {
            let expr: ast::Expr;
            let stmt: ast::Stmt;
            let stmt2: ast::Stmt;
            if self.check_kind(TokenKind::OpenParen) {
                expr = self.parse_expression();

                if self.check_kind(TokenKind::CloseParen) {
                    stmt = self.parse_statement();

                    if self.check_kind(TokenKind::Else) {
                        stmt2 = self.parse_statement();
                        return ast::Stmt::If {
                            cond: expr,
                            then_: Box::new(stmt),
                            else_: Some(Box::new(stmt2)),
                        };
                    } else {
                        return ast::Stmt::If {
                            cond: expr,
                            then_: Box::new(stmt),
                            else_: None,
                        };
                    }
                } else {
                    println!("Parser error: Parenthese not closed");
                    exit(-1);
                }
            }
        }

        println!("Parser error: 'if' keyword is missing (source: parse_if_statement())");
        exit(-1);
    }

    fn parse_iteration_statement(&mut self) -> ast::Stmt {
        let expr: ast::Expr;
        let stmt: ast::Stmt;
        if self.check_kind(TokenKind::While) {
            if self.check_kind(TokenKind::OpenParen) {
                expr = self.parse_expression();
                if self.check_kind(TokenKind::CloseParen) {
                    stmt = self.parse_statement();
                } else {
                    println!("Parser error: Parenthese not closed");
                    exit(-1);
                }

                return ast::Stmt::While {
                    cond: expr,
                    body: Box::new(stmt),
                };
            } else {
                println!("Parser error: Expected '(' after the 'while' keyword");
                exit(-1);
            }
        } else if self.check_kind(TokenKind::For) {
            let first: ast::Expr;
            let firstvar: Vec<(String, Option<ast::Expr>)>;
            let mut second: ast::Expr = ast::Expr::Literal(ast::Literal::Undefined);
            let mut third: ast::Expr = ast::Expr::Literal(ast::Literal::Undefined);
            let mut body: ast::Stmt = ast::Stmt::Empty;

            if self.check_kind(TokenKind::OpenParen) {
                // if ExpressionNoIn opt ;
                if self.check_kind(TokenKind::Var) {
                    firstvar = self.parse_variable_declaration_list();

                    if self.check_kind(TokenKind::SemiColon) {
                        if !self.check_kind(TokenKind::SemiColon) {
                            third = ast::Expr::Empty;
                            if !self.check_kind(TokenKind::CloseParen) {
                                println!("Parser error: Expected ')' after '('");
                                exit(-1);
                            }

                            body = self.parse_statement();
                        }
                        second = self.parse_expression();
                    } else if self.check_kind(TokenKind::In) {
                        second = self.parse_expression();

                        if !self.check_kind(TokenKind::CloseParen) {
                            println!("Parser error: Expected ')' after '('");
                            exit(-1);
                        }

                        body = self.parse_statement();
                    }

                    return ast::Stmt::For {
                        init: Some(ast::ForInit::Var(firstvar)),
                        cond: Some(second),
                        update: Some(third),
                        body: Box::new(body),
                    };
                } else if self.check_kind(TokenKind::New) {
                    first = self.parse_lefthand_side_expression();

                    if self.check_kind(TokenKind::In) {
                        second = self.parse_expression();
                    } else {
                        println!("Parser error: Expected 'in' after a lefthand sided expression in a 'for' loop");
                        exit(-1);
                    }

                    if !self.check_kind(TokenKind::CloseParen) {
                        println!("Parser error: Expected ')' after '('");
                        exit(-1);
                    }

                    return ast::Stmt::For {
                        init: Some(ast::ForInit::Expr(first)),
                        cond: Some(second),
                        update: Some(third),
                        body: Box::new(body),
                    };
                } else {
                    self.allow_in = false;
                    first = self.parse_expression(); // for (ExpressionNoIn opt; Expression opt ; Expression opt ) Statement

                    if self.check_kind(TokenKind::SemiColon) {
                        if !self.check_kind(TokenKind::SemiColon) {
                            third = ast::Expr::Empty;
                            if !self.check_kind(TokenKind::CloseParen) {
                                println!("Parser error: Expected ')' after '('");
                                exit(-1);
                            }
                        }
                        second = self.parse_expression();
                    }

                    body = self.parse_statement();

                    return ast::Stmt::For {
                        init: Some(ast::ForInit::Expr(first)),
                        cond: Some(second),
                        update: Some(third),
                        body: Box::new(body),
                    };
                }
            } else {
                println!("Parser error: Expected '(' after the 'for' keyword");
                exit(-1);
            }
        } else {
            println!("Parser error: No more options for iteration statement");
            exit(-1);
        }
    }

    // NOTE: The grammar does not look logical
    // ContinueStatement :
    //  continue [no LineTerminator here] Identifier_opt ;
    fn parse_continue_statement(&mut self) -> ast::Stmt {
        if self.check_kind(TokenKind::Continue) {
            return ast::Stmt::Continue;
        }

        println!(
            "Parser error: Expected 'continue' but found '{}'",
            self.peek().content
        );
        exit(-1);
    }

    // NOTE: The grammar does not look logical
    // BreakStatement :
    //  break [no LineTerminator here] Identifier_opt ;
    fn parse_break_statement(&mut self) -> ast::Stmt {
        if self.check_kind(TokenKind::Break) {
            return ast::Stmt::Break;
        }

        println!(
            "Parser error: Expected 'break' but found '{}'",
            self.peek().content
        );
        exit(-1);
    }

    // NOTE: The grammar does not look logical
    // ReturnStatement :
    //  return [no LineTerminator here] Identifier_opt ;
    fn parse_return_statement(&mut self) -> ast::Stmt {
        let expr: ast::Expr;

        if self.check_kind(TokenKind::Return) {
            if self.check_kind(TokenKind::SemiColon) || self.check_kind(TokenKind::NewLine) {
                return ast::Stmt::Return(None);
            }
            expr = self.parse_expression();

            return ast::Stmt::Return(Some(expr));
        }

        println!(
            "Parser error: Expected 'return' but found '{}'",
            self.peek().content
        );
        exit(-1);
    }

    fn parse_with_statement(&mut self) -> ast::Stmt {
        assert!(self.check_kind(TokenKind::With));
        self.advance();

        if !self.check_kind(TokenKind::OpenParen) {
            println!(
                "Parser error: Expected '(' but found '{}'",
                self.peek().content
            );
            exit(-1);
        }
        self.advance();

        let expr = self.parse_expression();

        if !self.check_kind(TokenKind::CloseParen) {
            println!(
                "Parser error: Expected ')' but found '{}'",
                self.peek().content
            );
            exit(-1);
        }
        self.advance();

        let stmt = self.parse_statement();

        return ast::Stmt::With {
            expr: expr,
            body: Box::new(stmt),
        };
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> ast::Program {
        self.tokens = tokens;
        self.pos = 0;

        let mut body = Vec::new();

        while self.peek().kind != TokenKind::EOF {
            if self.peek().kind == TokenKind::Function {
                body.push(ast::Stmt::Function(self.parse_function_declaration()));
            } else {
                body.push(self.parse_statement());
            }
        }

        ast::Program { body }
    }
}
