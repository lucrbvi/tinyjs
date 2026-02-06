use tinyjs::ast;
use tinyjs::lexer;
use tinyjs::parser;

fn parse_program(source: &str) -> ast::Program {
    let mut lex = lexer::Lexer {
        source: source.to_string(),
        cursor: lexer::Cursor { row: 0, line: 0 },
        line: 0,
        row: 0,
        prev_cr: false,
    };
    let tokens = lex.walk();
    let mut parser = parser::Parser {
        tokens: Vec::new(),
        pos: 0,
        allow_in: true,
        source: source.to_string(),
    };
    parser.parse(tokens)
}

fn first_stmt(source: &str) -> ast::Stmt {
    let program = parse_program(source);
    program.body.into_iter().next().expect("missing stmt")
}

fn expect_stmt(source: &str, label: &str, check: impl FnOnce(&ast::Stmt) -> bool) {
    let stmt = first_stmt(source);
    if !check(&stmt) {
        panic!("{}: unexpected stmt: {:?}", label, stmt);
    }
}

fn first_expr_from_expr_stmt(source: &str) -> ast::Expr {
    match first_stmt(source) {
        ast::Stmt::Expr(expr) => expr,
        other => panic!("expected Expr stmt, got {:?}", other),
    }
}

fn expect_expr(source: &str, label: &str, check: impl FnOnce(&ast::Expr) -> bool) {
    let expr = first_expr_from_expr_stmt(source);
    if !check(&expr) {
        panic!("{}: unexpected expr: {:?}", label, expr);
    }
}

#[test]
fn parses_empty_statement() {
    expect_stmt(";", "empty statement", |stmt| matches!(stmt, ast::Stmt::Empty));
}

#[test]
fn parses_block_statement() {
    expect_stmt("{}", "block statement", |stmt| matches!(stmt, ast::Stmt::Block(_)));
}

#[test]
fn parses_var_statement() {
    expect_stmt("var a = 1;", "var statement", |stmt| matches!(stmt, ast::Stmt::Var(_)));
}

#[test]
fn parses_if_statement() {
    expect_stmt("if (true) ;", "if statement", |stmt| {
        matches!(stmt, ast::Stmt::If { .. })
    });
}

#[test]
fn parses_if_else_statement() {
    expect_stmt("if (true) ; else ;", "if else statement", |stmt| {
        matches!(stmt, ast::Stmt::If { else_: Some(_), .. })
    });
}

#[test]
fn parses_while_statement() {
    expect_stmt("while (true) ;", "while statement", |stmt| {
        matches!(stmt, ast::Stmt::While { .. })
    });
}

#[test]
fn parses_for_statement() {
    expect_stmt(
        "for (i = 0; i < 1; i++) ;",
        "for statement",
        |stmt| matches!(stmt, ast::Stmt::For { .. }),
    );
}

#[test]
fn parses_for_var_statement() {
    expect_stmt(
        "for (var i = 0; i < 1; i++) ;",
        "for var statement",
        |stmt| matches!(stmt, ast::Stmt::For { .. }),
    );
}

#[test]
fn parses_for_in_statement() {
    expect_stmt("for (i in obj) ;", "for in statement", |stmt| {
        matches!(stmt, ast::Stmt::ForIn { .. })
    });
}

#[test]
fn parses_continue_statement() {
    expect_stmt("continue;", "continue statement", |stmt| {
        matches!(stmt, ast::Stmt::Continue)
    });
}

#[test]
fn parses_break_statement() {
    expect_stmt("break;", "break statement", |stmt| {
        matches!(stmt, ast::Stmt::Break)
    });
}

#[test]
fn parses_return_statement() {
    expect_stmt("return 1;", "return statement", |stmt| {
        matches!(stmt, ast::Stmt::Return(Some(_)))
    });
}

#[test]
fn parses_return_asi_statement() {
    expect_stmt("return\n1;", "return ASI statement", |stmt| {
        matches!(stmt, ast::Stmt::Return(None))
    });
}

#[test]
fn parses_with_statement() {
    expect_stmt("with (obj) ;", "with statement", |stmt| {
        matches!(stmt, ast::Stmt::With { .. })
    });
}

#[test]
fn parses_function_declaration() {
    expect_stmt(
        "function f(a, b) { return a; }",
        "function declaration statement",
        |stmt| matches!(stmt, ast::Stmt::Function(_)),
    );
}

#[test]
fn parses_expression_statement() {
    expect_stmt("a + b;", "expression statement", |stmt| {
        matches!(stmt, ast::Stmt::Expr(_))
    });
}

#[test]
fn parses_operator_precedence_mul_over_add() {
    expect_expr("1 + 2 * 3;", "precedence: mul over add", |expr| {
        if let ast::Expr::Binary { op: ast::BinOp::Add, right, .. } = expr {
            matches!(&**right, ast::Expr::Binary { op: ast::BinOp::Mul, .. })
        } else {
            false
        }
    });
}

#[test]
fn parses_operator_associativity_left() {
    expect_expr("1 - 2 - 3;", "associativity: left", |expr| {
        if let ast::Expr::Binary { op: ast::BinOp::Sub, left, .. } = expr {
            matches!(&**left, ast::Expr::Binary { op: ast::BinOp::Sub, .. })
        } else {
            false
        }
    });
}

#[test]
fn parses_ternary_expression() {
    expect_expr("a ? b : c;", "ternary", |expr| {
        matches!(expr, ast::Expr::Ternary { .. })
    });
}

#[test]
fn parses_sequence_expression() {
    expect_expr("a, b, c;", "sequence", |expr| {
        matches!(expr, ast::Expr::Sequence(v) if v.len() == 3)
    });
}

#[test]
fn parses_assignment_expression() {
    expect_expr("a += 1;", "assignment op", |expr| {
        matches!(expr, ast::Expr::Assign { op: ast::AssignOp::AddAssign, .. })
    });
}

#[test]
fn parses_unary_expression() {
    expect_expr("!a;", "unary not", |expr| {
        matches!(expr, ast::Expr::Unary { op: ast::UnaryOp::Not, .. })
    });
}

#[test]
fn parses_update_prefix_expression() {
    expect_expr("++a;", "update prefix", |expr| {
        matches!(expr, ast::Expr::Update { prefix: true, op: ast::UpdateOp::Inc, .. })
    });
}

#[test]
fn parses_update_postfix_expression() {
    expect_expr("a--;", "update postfix", |expr| {
        matches!(expr, ast::Expr::Update { prefix: false, op: ast::UpdateOp::Dec, .. })
    });
}

#[test]
fn parses_member_and_index_expression() {
    expect_expr("obj.a[b];", "member+index", |expr| {
        if let ast::Expr::Index { object, .. } = expr {
            matches!(&**object, ast::Expr::Member { .. })
        } else {
            false
        }
    });
}

#[test]
fn parses_call_expression() {
    expect_expr("f(a, b);", "call expression", |expr| {
        matches!(expr, ast::Expr::Call { .. })
    });
}

#[test]
fn parses_new_expression() {
    expect_expr("new F(a);", "new expression", |expr| {
        matches!(expr, ast::Expr::New { .. })
    });
}

#[test]
fn parses_array_literal() {
    expect_expr("[1, 2, 3];", "array literal", |expr| {
        matches!(expr, ast::Expr::Literal(ast::Literal::Array(v)) if v.len() == 3)
    });
}

#[test]
fn parses_object_literal() {
    expect_expr("{a: 1, \"b\": 2, 3: 4};", "object literal", |expr| {
        matches!(expr, ast::Expr::Literal(ast::Literal::Object(v)) if v.len() == 3)
    });
}

#[test]
fn parses_function_expression() {
    expect_expr("function(a, b) { return a; };", "function expression", |expr| {
        matches!(expr, ast::Expr::Function(_))
    });
}

#[test]
fn parses_nested_statement_block() {
    expect_stmt("{ if (true) { return 1; } }", "nested block", |stmt| {
        matches!(stmt, ast::Stmt::Block(v) if v.len() == 1)
    });
}

#[test]
fn parses_asi_after_break() {
    expect_stmt("break\n;", "break ASI", |stmt| matches!(stmt, ast::Stmt::Break));
}

#[test]
fn parses_for_in_with_var_initializer() {
    expect_stmt("for (var i = 0 in obj) ;", "for-in with var init", |stmt| {
        matches!(stmt, ast::Stmt::ForIn { .. })
    });
}
