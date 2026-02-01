use tinyjs::lexer;
use tinyjs::parser;

fn main() {
    let mut lex = lexer::Lexer {
        source: "var a = function() {return 1e3.toString()}".to_string(),
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
