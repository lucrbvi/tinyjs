# TinyJS

A simple interpreter for JavaScript 1.1 / ECMAScript 1 in Rust without dependencies.

The pipeline is working like this:
1. The lexer analyze the source code
2. The parser generate the AST
3. IR generation and optimization
4. Compilation to bytecode
5. The VM execute the compiled code

[~] ECMA-262, first edition (1997) [link](https://www.ecma-international.org/wp-content/uploads/ECMA-262_1st_edition_june_1997.pdf)

## LLM usage for code

I’ve used Codex to assist me. It’s been especially helpful for writing test cases and spotting inconsistencies in the codebase.
I still wrote every line of code myself (before cleaning and optimizing things).

I recommend [this blog article](https://mitchellh.com/writing/my-ai-adoption-journey) to understand how LLM assistance can be used without losing control of a technical project.
