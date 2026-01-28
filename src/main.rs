use tinyjs::lexer;

fn main() {
    let mut lex = lexer::Lexer{
        source: "var a = 1_000.02; console.log(a);".to_string(),
        cursor: lexer::Cursor{row: 0, line:0},
        line: 0,
        row: 0,
    };

    let tokens = lex.walk();
    for token in tokens {
        println!("{}", token.content);
    }
}
