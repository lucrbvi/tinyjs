use tinyjs::lexer;
use tinyjs::parser;

fn main() {
    let source = "var i=0; while(i++<5){if (i==4) {break;} console.log('hi')} var b = {a: 16.2}; var c = undefined; var d = !{}\nfunction nen() {\n return 15-2;\n};".to_string();
    let mut lex = lexer::Lexer {
        source: source.clone(),
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
        source,
    };

    let program = parser.parse(tokens);

    for stmt in program.body {
        println!("{:#?}", stmt);
    }
}
