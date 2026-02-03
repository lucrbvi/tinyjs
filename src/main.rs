use tinyjs::lexer;
use tinyjs::parser;

fn main() {
    let mut lex = lexer::Lexer {
        source: "var i=0; while(i++<5){console.log('Hello')} var b = {a: 16.2}; var c = undefined; var d = !{}".to_string(),
        cursor: lexer::Cursor { row: 0, line: 0 },
        line: 0,
        row: 0,
    };

    let tokens = lex.walk();

    let mut parser = parser::Parser {
        tokens: Vec::new(),
        pos: 0,
        allow_in: true,
    };

    let program = parser.parse(tokens);

    for stmt in program.body {
        println!("{:#?}", stmt);
    }
}
