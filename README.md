# TinyJS

A simple interpreter for JavaScript 1.1 / ECMAScript 1 in Rust without dependencies.

The pipeline is working like this:
1. The lexer analyze the source code
2. The parser generate the AST
3. IR generation and optimization
4. Compilation to bytecode
5. The VM execute the compiled code

[~] ECMA-262, first edition (1997) [link](https://www.ecma-international.org/wp-content/uploads/ECMA-262_1st_edition_june_1997.pdf)
