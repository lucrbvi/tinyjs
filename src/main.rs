use tinyjs::lexer;

fn main() {
    let mut lex = lexer::Lexer{
        source: "var a = 'Hi'; console.log(a);".to_string(),
        cursor: lexer::Cursor{row: 0, line:0},
        line: 0,
        row: 0,
    };

    let tokens = lex.walk();
    for token in tokens {
        println!("{}", token.content);
    }
}
