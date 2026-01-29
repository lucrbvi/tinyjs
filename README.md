# TinyJS

A simple and embeddable JavaScript interpreter written in Rust without third-party dependencies.
We expose our API to C ABI.

The pipeline is working like this:
1. The lexer analyze the source code
2. The parser generate the AST
3. Optimization and IR generation
4. Compilation
5. The VM execute the compiled version

We are implementing the language following the ECMA-262 standard.
To make things easier, we are implementing ECMAScript by following the oldest version and slowly upgrading it by reading the others editions.

[~] ECMA-262, first edition (1997) [link](https://www.ecma-international.org/wp-content/uploads/ECMA-262_1st_edition_june_1997.pdf)
[] ECMA-262, 2 (1998) 
[] ECMA-262, 3 (1999)
[] ECMA-262, 5 (2009)
[] ECMA-262, 5.1 (2011)
[] ECMA-262, 6 (2015)
[] ECMA-262, 7 (2016)
[] ECMA-262, 8 (2017)
[] ECMA-262, 9 (2018)
[] ECMA-262, 10 (2019)
[] ECMA-262, 11 (2020)
[] ECMA-262, 12 (2021)
[] ECMA-262, 13 (2022)
[] ECMA-262, 14 (2023)
[] ECMA-262, 15 (2024)
[] ECMA-262, 16 (2025)
[] ECMA-262, 17 (2026) (ES Next)
