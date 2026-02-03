// You should read this with the ECMAScript Third Edition on Annex B 
// (we ignore grammar on reserved keywords for ECMAScript first edition)

use crate::ast;
use crate::lexer::{Token, TokenKind};

use std::process::exit;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub allow_in: bool, // used to exclude parsing "in" in certain scenarios
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
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

    fn parse_string(&mut self, x: Token) -> String {
        if x.content.chars().nth(0) == Some('\'')
            || x.content.chars().nth(0) == Some('"') {
                // we could have done this in lexer but it's fine here too
                // (we drop the '' or "" in strings)
                let mut y = x.content.clone();
                y.pop();
                y.remove(0);
                return y;
        }

        return x.content.clone();
    }

    fn parse_primary_expression(&mut self) -> ast::Expr {
        let x = self.peek();
        match x.kind {
            TokenKind::This => {
                self.advance();
                return ast::Expr::This;
            }
            TokenKind::Undefined => {
                self.advance();
                return ast::Expr::Literal(ast::Literal::Undefined);
            }
            TokenKind::Identifier => {
                let name = x.content.clone();
                self.advance();
                return ast::Expr::Identifier(name);
            }
            TokenKind::String => {
                let cloned_x = x.clone();
                self.advance();
                return ast::Expr::Literal(ast::Literal::String(self.parse_string(cloned_x)));
            }
            TokenKind::True => {
                self.advance();
                return ast::Expr::Literal(ast::Literal::Bool(true));
            }
            TokenKind::False => {
                self.advance();
                return ast::Expr::Literal(ast::Literal::Bool(false));
            }
            TokenKind::Null => {
                self.advance();
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
                self.advance();
                return self.parse_object();
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
    }

    fn parse_object(&mut self) -> ast::Expr {
        if self.check_kind(TokenKind::CloseCurly) {
            return ast::Expr::Literal(ast::Literal::Object(vec![]));
        }

        let props = self.parse_property_name_and_value_list();

        if !self.check_kind(TokenKind::CloseCurly) {
            println!("Parser error: expected '}}' after object");
            exit(-1);
        }

        return ast::Expr::Literal(ast::Literal::Object(props));
    }

    fn parse_property_name_and_value_list(&mut self) -> Vec<(ast::PropertyKey, ast::Expr)> {
        let mut outvec: Vec<(ast::PropertyKey, ast::Expr)> = vec![];

        loop {
            let property_name: ast::PropertyKey;
            if self.peek().kind == TokenKind::String {
                property_name = ast::PropertyKey::String(self.parse_string(self.peek().clone()));
                self.advance();
            } else if self.peek().kind == TokenKind::Number {
                property_name = ast::PropertyKey::Number(self.peek().content.clone().parse().unwrap());
                self.advance();
            } else if self.peek().kind == TokenKind::Identifier {
                property_name = ast::PropertyKey::Identifier(self.parse_identifier());
            } else {
                println!("Parser error: Expected a String or a Number or an Identifier but found '{}' of type {:#?}", self.peek().content, self.peek().kind);
                exit(-1);
            }

            if !self.check_kind(TokenKind::DoubleDot) {
                println!("Parser error: Expected ':' in object but found '{}'", self.peek().content);
                exit(-1);
            }

            let assignment_expr = self.parse_assignment_expression();

            outvec.push((property_name, assignment_expr));

            if !self.check_kind(TokenKind::Comma) {
                break;
            }
        }

        return outvec;
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
        let mut expr = self.parse_logical_and_expression();

        while self.peek().kind == TokenKind::Or {
            self.advance();
            let right = self.parse_logical_and_expression();
            expr = ast::Expr::Binary {
                op: ast::BinOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn parse_logical_and_expression(&mut self) -> ast::Expr {
        let mut expr = self.parse_bitwise_or_expression();

        while self.peek().kind == TokenKind::And {
            self.advance();
            let right = self.parse_bitwise_or_expression();
            expr = ast::Expr::Binary {
                op: ast::BinOp::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn parse_bitwise_or_expression(&mut self) -> ast::Expr {
        let mut expr = self.parse_bitwise_xor_expression();

        while self.peek().kind == TokenKind::Bar {
            self.advance();
            let right = self.parse_bitwise_xor_expression();
            expr = ast::Expr::Binary {
                op: ast::BinOp::BitOr,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn parse_bitwise_xor_expression(&mut self) -> ast::Expr {
        let mut expr = self.parse_bitwise_and_expression();

        while self.peek().kind == TokenKind::Caret {
            self.advance();
            let right = self.parse_bitwise_and_expression();
            expr = ast::Expr::Binary {
                op: ast::BinOp::BitXor,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn parse_bitwise_and_expression(&mut self) -> ast::Expr {
        let mut expr = self.parse_equality_expression();

        while self.peek().kind == TokenKind::Ampersand {
            self.advance();
            let right = self.parse_equality_expression();
            expr = ast::Expr::Binary {
                op: ast::BinOp::BitAnd,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn parse_equality_expression(&mut self) -> ast::Expr {
        let mut expr = self.parse_relational_expression();

        loop {
            let op = match self.peek().kind {
                TokenKind::DoubleEqual => ast::BinOp::Eq,
                TokenKind::NotEqual => ast::BinOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_relational_expression();
            expr = ast::Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn parse_relational_expression(&mut self) -> ast::Expr {
        let mut expr = self.parse_shift_expression();

        loop {
            let op = match self.peek().kind {
                TokenKind::LessThan => ast::BinOp::Lt,
                TokenKind::GreaterThan => ast::BinOp::Gt,
                TokenKind::GreaterThanEqual => ast::BinOp::Ge,
                TokenKind::LessThanEqual => ast::BinOp::Le,
                TokenKind::In if self.allow_in => ast::BinOp::In,
                _ => break,
            };
            self.advance();
            let right = self.parse_shift_expression();
            expr = ast::Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn parse_shift_expression(&mut self) -> ast::Expr {
        let mut expr = self.parse_additive_expression();

        loop {
            let op = match self.peek().kind {
                TokenKind::LeftShift => ast::BinOp::Shl,
                TokenKind::RightShift => ast::BinOp::Shr,
                TokenKind::TripleGreaterThan => ast::BinOp::UShr,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive_expression();
            expr = ast::Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn parse_additive_expression(&mut self) -> ast::Expr {
        let mut expr = self.parse_multiplicative_expression();

        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => ast::BinOp::Add,
                TokenKind::Minus => ast::BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative_expression();
            expr = ast::Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn parse_multiplicative_expression(&mut self) -> ast::Expr {
        let mut expr = self.parse_unary_expression();

        loop {
            let op = match self.peek().kind {
                TokenKind::Asterisk => ast::BinOp::Mul,
                TokenKind::Slash => ast::BinOp::Div,
                TokenKind::Modulo => ast::BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary_expression();
            expr = ast::Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
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
        let mut expr = self.parse_new_expression();

        loop {
            match self.peek().kind {
                TokenKind::OpenParen => {
                    self.advance();
                    let args = self.parse_arguments();
                    expr = ast::Expr::Call {
                        callee: Box::new(expr),
                        args: Box::new(args),
                    };
                }
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
                self.consume_semicolon_or_insert();
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
            self.consume_semicolon_or_insert();
            return ast::Stmt::Var(vars);
        }
        println!("Parser error: 'var' expected but not found in parse_variable_statement()");
        exit(-1);
    }

    fn parse_variable_declaration_list(&mut self) -> Vec<(String, Option<ast::Expr>)> {
        let mut vars: Vec<(String, Option<ast::Expr>)> = vec![];

        if self.peek().kind != TokenKind::Identifier {
            return vars;
        }

        loop {
            let name: String = self.peek().content.clone();
            let mut init: ast::Expr = ast::Expr::Literal(ast::Literal::Undefined);
            self.advance();

            if self.check_kind(TokenKind::Equal) {
                init = self.parse_assignment_expression();
            }

            vars.push((name, Some(init)));

            if !self.check_kind(TokenKind::Comma) {
                break;
            }
            if self.peek().kind != TokenKind::Identifier {
                println!("Parser error: expected identifier after ',' in variable declaration");
                exit(-1);
            }
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
        let mut stmt: ast::Stmt = ast::Stmt::Empty;
        if self.check_kind(TokenKind::While) {
            if self.check_kind(TokenKind::OpenParen) {
                expr = self.parse_expression();
                if self.check_kind(TokenKind::CloseParen) {
                    stmt = self.parse_statement();
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

                    if self.check_kind(TokenKind::In) {
                        if firstvar.len() != 1 {
                            println!("Parser error: expected a single variable in 'for...in'");
                            exit(-1);
                        }
                        let name = firstvar[0].0.clone();
                        second = self.parse_expression();

                        if !self.check_kind(TokenKind::CloseParen) {
                            println!("Parser error: Expected ')' after '('");
                            exit(-1);
                        }

                        body = self.parse_statement();

                        return ast::Stmt::ForIn {
                            var: name,
                            expr: second,
                            body: Box::new(body),
                        };
                    }

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
                    let prev_allow_in = self.allow_in;
                    self.allow_in = false;
                    first = self.parse_expression(); // for (ExpressionNoIn opt; Expression opt ; Expression opt ) Statement
                    self.allow_in = prev_allow_in;

                    if self.check_kind(TokenKind::In) {
                        let name = match first {
                            ast::Expr::Identifier(n) => n,
                            _ => {
                                println!("Parser error: expected identifier before 'in' in 'for...in'");
                                exit(-1);
                            }
                        };
                        second = self.parse_expression();

                        if !self.check_kind(TokenKind::CloseParen) {
                            println!("Parser error: Expected ')' after '('");
                            exit(-1);
                        }

                        body = self.parse_statement();

                        return ast::Stmt::ForIn {
                            var: name,
                            expr: second,
                            body: Box::new(body),
                        };
                    }

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

    fn parse_continue_statement(&mut self) -> ast::Stmt {
        if self.check_kind(TokenKind::Continue) {
            self.consume_semicolon_or_insert();
            return ast::Stmt::Continue;
        }

        println!(
            "Parser error: Expected 'continue' but found '{}'",
            self.peek().content
        );
        exit(-1);
    }

    fn parse_break_statement(&mut self) -> ast::Stmt {
        if self.check_kind(TokenKind::Break) {
            self.consume_semicolon_or_insert();
            return ast::Stmt::Break;
        }

        println!(
            "Parser error: Expected 'break' but found '{}'",
            self.peek().content
        );
        exit(-1);
    }

    fn parse_return_statement(&mut self) -> ast::Stmt {
        let expr: ast::Expr;

        if self.check_kind(TokenKind::Return) {
            if self.peek().kind == TokenKind::SemiColon
                || self.peek().kind == TokenKind::CloseCurly
                || self.peek().kind == TokenKind::EOF
                || self.peek().line_terminator_before
            {
                self.consume_semicolon_or_insert();
                return ast::Stmt::Return(None);
            }
            expr = self.parse_expression();

            self.consume_semicolon_or_insert();
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

    fn consume_semicolon_or_insert(&mut self) {
        if self.check_kind(TokenKind::SemiColon) {
            return;
        }
        if self.peek().kind == TokenKind::CloseCurly
            || self.peek().kind == TokenKind::EOF
            || self.peek().line_terminator_before
        {
            return;
        }
        println!("Parser error: expected ';'");
        exit(-1);
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
