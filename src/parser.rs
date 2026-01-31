#![allow(dead_code)]

// You should read this with the ECMAScript Third Edition on Annex B

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

            if !self.check_kind(TokenKind::Comma) {
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
                let next = self.peek_n(1);
                if next.kind == TokenKind::Identifier
                    && (next.content == "E" || next.content == "e")
                {
                    self.advance();
                    let next2 = self.peek_n(1);
                    if next2.kind == TokenKind::Number {
                        // numbers with scientific notation
                        // example: 1e3, 1.2e8.32
                        // we calculate the result at compile time right here
                        let left_exp: f64 = x_content.parse().unwrap();
                        let right_exp: f64 = next2.content.parse().unwrap();
                        return ast::Expr::Literal(ast::Literal::Number(
                            left_exp * 10_f64.powf(right_exp),
                        ));
                    }
                }
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
                let y = self.peek();
                if y.kind != TokenKind::CloseParen {
                    println!(
                        "Parser error: Unexpected token '{}', expected ')')'",
                        y.content
                    );
                    exit(-1);
                }
                self.advance();
                return expr;
            }
            _ => {}
        }
        return ast::Expr::This;
    }

    fn parse_array(&mut self) -> ast::Expr {
        let mut elements: Vec<ast::Expr> = vec![];

        while !self.check_kind(TokenKind::CloseBracket) {
            if self.check_kind(TokenKind::Comma) {
                elements.push(ast::Expr::Literal(ast::Literal::Undefined));
            } else {
                elements.push(self.parse_assignment_expression());
            }

            if !self.match_(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        if !self.check_kind(TokenKind::CloseBracket) {
            println!("Parser error: expected ']'");
            exit(-1);
        }
        self.advance();

        ast::Expr::Literal(ast::Literal::Array(elements))
    }

    fn parse_assignment_expression(&mut self) -> ast::Expr {
        let left = self.parse_conditional_expression();

        if self.check_kind(TokenKind::Equal)
            || self.check_kind(TokenKind::PlusEqual)
            || self.check_kind(TokenKind::MinusEqual)
            || self.check_kind(TokenKind::AsteriskEqual)
            || self.check_kind(TokenKind::SlashEqual)
            || self.check_kind(TokenKind::ModuloEqual)
            || self.check_kind(TokenKind::LeftShiftEqual)
            || self.check_kind(TokenKind::RightShiftEqual)
            || self.check_kind(TokenKind::TripleGreaterThanEqual)
            || self.check_kind(TokenKind::AmpersandEqual)
            || self.check_kind(TokenKind::CaretEqual)
            || self.check_kind(TokenKind::BarEqual)
        {
            // LeftHandSideExpression AssignmentOperator AssignmentExpression
            let assignement_op = self.parse_assignment_operator();
            self.advance();
            let expr = self.parse_assignment_expression();

            return ast::Expr::Assign {
                target: Box::new(left),
                op: assignement_op,
                value: Box::new(expr),
            };
        }

        return left; // ConditionalExpression
    }

    fn parse_assignment_operator(&mut self) -> ast::AssignOp {
        let x = self.peek();

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

        if self.peek().kind == TokenKind::Question {}

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

        if self.check_kind(TokenKind::OpenParen) {
            return self.parse_call_expression();
        }

        return expr;
    }

    fn parse_new_expression(&mut self) -> ast::Expr {
        return self.parse_member_expression();
    }

    fn parse_call_expression(&mut self) -> ast::Expr {
        let member_expr = self.parse_member_expression();

        if self.check_kind(TokenKind::OpenParen) {
            let args = self.parse_arguments();
            return ast::Expr::Call {
                callee: Box::new(member_expr),
                args: Box::new(args),
            };
        } else if self.check_kind(TokenKind::OpenBracket) {
            let args = self.parse_expression();
            return ast::Expr::Index {
                object: Box::new(member_expr),
                index: Box::new(args),
            };
        } else if self.check_kind(TokenKind::Dot) {
            let id = self.parse_identifier();
            return ast::Expr::Member {
                object: Box::new(member_expr),
                property: id,
            };
        }

        println!("Parser error: Illegal token '{}'", self.peek().content);
        exit(-1);
    }

    fn parse_arguments(&mut self) -> ast::Expr {
        if self.check_kind(TokenKind::CloseParen) {
            return ast::Expr::Sequence(vec![]);
        }
        let list = self.parse_argument_list();
        return ast::Expr::Sequence(list);
    }

    fn parse_argument_list(&mut self) -> Vec<ast::Expr> {
        let mut outvec = vec![];
        while !self.check_kind(TokenKind::CloseParen) {
            outvec.push(self.parse_assignment_expression());
        }
        return outvec;
    }

    fn parse_member_expression(&mut self) -> ast::Expr {
        let expr: ast::Expr;

        if self.check_kind(TokenKind::Function) {
            expr = self.parse_function_expression();
        } else if self.check_kind(TokenKind::New) {
            expr = self.parse_member_expression();
            let args = self.parse_arguments();

            return ast::Expr::New {
                callee: Box::new(expr),
                args: Box::new(args),
            };
        } else {
            expr = self.parse_primary_expression();
        }

        if expr == ast::Expr::Literal(ast::Literal::Undefined) {
            println!("Parser error: illegal token '{}'", self.peek().content);
            exit(-1);
        }

        if self.check_kind(TokenKind::OpenBracket) {
            let args = self.parse_expression();
            if !self.check_kind(TokenKind::CloseBracket) {
                println!("Parser error: illegal token '{}'", self.peek().content);
                exit(-1);
            }

            return ast::Expr::Index {
                object: Box::new(expr),
                index: Box::new(args),
            };
        } else if self.check_kind(TokenKind::Dot) {
            let args = self.parse_arguments();
            return ast::Expr::Call {
                callee: Box::new(expr),
                args: Box::new(args),
            };
        }

        println!("Parser error: illegal token '{}'", self.peek().content);
        exit(-1);
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

        let mut params: Vec<String> = vec![];
        if self.check_kind(TokenKind::OpenParen) {
            params = self.parse_parameter_list();
            if !self.check_kind(TokenKind::CloseParen) {
                println!("Parser error: Not found ')' after '('");
                exit(-1);
            }
            self.advance();
        }

        let mut body: Vec<ast::Stmt> = vec![];
        if self.check_kind(TokenKind::OpenCurly) {
            body = self.parse_function_body();
            if !self.check_kind(TokenKind::CloseCurly) {
                println!("Parser error: Not found '}}' after '{{'");
                exit(-1);
            }
            self.advance();
        }

        ast::Expr::Function(ast::Function { name, params, body })
    }

    fn parse_parameter_list(&mut self) -> Vec<String> {
        let mut outvec = vec![];
        self.advance();
        while !self.check_kind(TokenKind::CloseParen) {
            outvec.push(self.parse_identifier());
            if !self.check_kind(TokenKind::CloseParen) {
                self.advance();
            }
        }
        return outvec;
    }

    fn parse_function_body(&mut self) -> Vec<ast::Stmt> {
        let mut body = vec![];
        self.advance();
        while !self.check_kind(TokenKind::CloseCurly) {
            body.push(self.parse_statement());
        }
        return body;
    }

    fn parse_function_declaration(&mut self) -> ast::Function {
        let name: String = self.parse_identifier();

        let mut params: Vec<String> = vec![];
        if self.check_kind(TokenKind::OpenParen) {
            params = self.parse_parameter_list();
            if !self.check_kind(TokenKind::CloseParen) {
                println!("Parser error: Not found ')' after '('");
                exit(-1);
            }
            self.advance();
        }

        let mut body: Vec<ast::Stmt> = vec![];
        if self.check_kind(TokenKind::OpenCurly) {
            body = self.parse_function_body();
            if !self.check_kind(TokenKind::CloseCurly) {
                println!("Parser error: Not found '}}' after '{{'");
                exit(-1);
            }
            self.advance();
        }

        return ast::Function { 
            name: Some(name), 
            params: params, 
            body: body,
        };
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
                return ast::Stmt::Empty;
            }
            TokenKind::Var => {
                return self.parse_variable_statement();
            }
            TokenKind::If => {
                return self.parse_if_statement();
            }
            TokenKind::Do
            |TokenKind::While
            |TokenKind::For => {
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
            TokenKind::Switch => {
                return self.parse_switch_statement();
            }
            TokenKind::Throw => {
                return self.parse_throw_statement();
            }
            TokenKind::Try => {
                return self.parse_try_statement();
            }
            _ => {
                // Not Function
                return ast::Stmt::Expr(self.parse_expression());
            },
        }
    }

    fn parse_block(&mut self) -> ast::Stmt {
        if self.check_kind(TokenKind::OpenCurly) {
            self.advance();
            if self.check_kind(TokenKind::CloseCurly) {
                return ast::Stmt::Block(vec![]);
            }

            let stmts = self.parse_statement_list();

            if !self.check_kind(TokenKind::CloseCurly) {
                println!("Parser error: expected '}}'");
                exit(-1);
            }
            self.advance();
            return ast::Stmt::Block(stmts);
        }
        ast::Stmt::Block(vec![])
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
            return ast::Stmt::Var(self.parse_variable_declaration_list());
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
                        return ast::Stmt::If{
                            cond: expr,
                            then_: Box::new(stmt),
                            else_: Some(Box::new(stmt2)),
                        };
                    } else {
                        return ast::Stmt::If{
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

                return ast::Stmt::While{
                    cond: expr,
                    body: Box::new(stmt),
                };
            } else {
                println!("Parser error: Expected '(' after the 'while' keyword");
                exit(-1);
            }
        } else if self.check_kind(TokenKind::For) {
            let first: ast::Expr;
            let firstvar: Vec<(String, Option<Expr>)>;
            let second: ast::Expr;
            let third: ast::Expr;
            let body: ast::Stmt;

            if self.check_kind(TokenKind::OpenParen) {
                // if ExpressionNoIn opt ;
                if self.check_kind(TokenKind::Var) {
                    firstvar = self.parse_variable_declaration_list();

                    if self.check_kind(TokenKind::SemiColon) {
                        if !self.check_kind(TokenKind::SemiColon) {
                            second = ast::Expr::Empty;
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

                    return ast::Stmt::For{
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

                    return ast::Stmt::For{
                        init: Some(ast::ForInit::Expr(first)),
                        cond: Some(second),
                        update: Some(third),
                        body: Box::new(body),
                    };
                } else {
                    println!("Parser error: parse_iteration_statement() is not finished, you may used a not implemented feature or you just did some illegal stuff");
                    exit(-1);
                }
            } else {
                println!("Parser error: Expected '(' after the 'for' keyword");
                exit(-1);
            }
        }
    }
}
