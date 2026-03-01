/*
 * TinyJS's IR is a simple Three-Adress Code-ish. Here are cool features:
 *  - 'Label' as a function: easier to optimize jumps and constant folding
 *  - Arrays are dynamic and are used to represent objects and arrays (you can delete a key by setting it to Undefined)
 *  - Prototypes properties are injected in an object by the AST->IR compiler
 *
 *  The AST->IR compiler implement most of the rules of ES1 seen in the standard paper
 *
 * Here is an example of a program in our IR
 * JS version: 'function add(a, b) { return a+b }; add(5, 2)'
 * IR version:
 *  FnDeclare("add", 2)
 *      Return(Add(arg[0], arg[1]))
 *  FnEnd()
 *  FnCall([5, 2])
 */

use crate::ast;
use crate::error;

#[derive(Debug, Clone)]
pub enum Operand {
    Var(String),  // reference to another variable
    Const(Const), // initial value
}

pub struct Program {
    pub body: Vec<Instruction>,
}

// JS Objects are destructured in multiple variables
// Ex: var a = {b: 15, c: "Hey"}; -> a_b = 15; a_c = "Hey";
#[derive(Debug, Clone)]
pub enum Const {
    String(String),
    Number(f64),
    Boolean(bool),
    Undefined,
    Null,
}

// A Function take one or two arguments (Nodes) and
// assign its result to the target node in an instruction
#[derive(Debug)]
pub enum Function {
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mul(Operand, Operand),
    Mod(Operand, Operand),
    Div(Operand, Operand),
    Pow(Operand, Operand),
    Inv(Operand), // return the inverse of a boolean (true -> false ; false -> true)
    Equal(Operand, Operand), // a == b
    NotEqual(Operand, Operand), // a != b
    LessThan(Operand, Operand), // a < b
    GreaterThan(Operand, Operand), // a > b
    LessThanEqual(Operand, Operand), // a <= b
    GreaterThanEqual(Operand, Operand), // a >= b
}

// Functions that do not return anything
#[derive(Debug)]
pub enum SoloFunction {
    Label(i64), // create a "target" for jumps ; the "key" is it's only argument, must be a Number
    JumpIf(Operand, i64), // jump to a label if the argument is true, only accept Boolean
    Jump(i64),
    Kill(Operand),        // dereference a var
    FnStart(String, i64), // declare the start of a function block (string = name, i64 = number of arguments)
    FnEnd(),              // end a function block
    Return(Option<Operand>),
    PushToScope(Operand),    // push an object in scope chain (for `with`)
    RemoveFromScope(),       // pop last pushed object from scope chain
    FnCall(String, Operand), // Operand = (...) with another operands inside (like a JS object)
    Call(Operand, Operand),  // Call the first operand as a function
}

#[derive(Debug)]
pub enum Instruction {
    Assign {
        // a = b
        dest: String,
        src: Operand,
    },
    Call {
        // label(1) ; jumpif(1==2, 1)
        function: SoloFunction, // function store the arguments
    },
    Classic {
        dest: String,
        function: Function, // function store the arguments
    },
}

// AST -> IR
pub struct Compiler {
    pub source: ast::Program,
    pub pos: usize,
    pub output: Program,
    pub label_stack: i64,
}

impl Compiler {
    pub fn compile(&mut self) {
        let body = std::mem::take(&mut self.source.body);
        for s in body {
            self.compile_stmt(s);
        }
    }

    fn error(&mut self, message: &'static str) -> ! {
        error::fail(format!("AST-> IR Compiler error: {}", message));
    }

    fn new_label(&mut self) -> i64 {
        self.label_stack += 1;
        self.label_stack
    }

    // big switch statement
    fn compile_stmt(&mut self, s: ast::Stmt) -> () {
        match s {
            ast::Stmt::Block(v) => {
                for vs in v {
                    self.compile_stmt(vs);
                }
            }
            ast::Stmt::Var(v) => {
                for var in v {
                    let e = match var.1 {
                        Some(expr) => self.compile_expr(expr),
                        None => Operand::Const(Const::Undefined),
                    };
                    self.output.body.push(Instruction::Assign {
                        dest: var.0,
                        src: e,
                    });
                }
            }
            ast::Stmt::Expr(e) => {
                self.compile_expr(e);
            }
            ast::Stmt::If { cond, then_, else_ } => {
                let entryl = self.new_label();
                let outl = self.new_label();

                let condop = self.compile_expr(cond);
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::JumpIf(condop, entryl),
                });
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(outl),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(entryl),
                });
                self.compile_stmt(*then_);

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(outl),
                });

                match else_ {
                    Some(els) => self.compile_stmt(*els),
                    None => (),
                };
            }

            ast::Stmt::Empty => {}
            _ => self.error("Unsupported statement type"),
        }
    }

    fn compile_expr(&mut self, _expr: ast::Expr) -> Operand {
        return Operand::Const(Const::Number(0.0));
    }
}
