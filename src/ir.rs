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

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Var(String),  // reference to another variable
    Const(Const), // initial value
}

pub struct Program {
    pub body: Vec<Instruction>,
}

// JS Objects are destructured in multiple variables
// Ex: var a = {b: 15, c: "Hey"}; -> a_b = 15; a_c = "Hey";
#[derive(Debug, Clone, PartialEq)]
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
    Noop(Operand), // return the operand
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
    PushToScope(Operand),        // push an object in scope chain (for `with`)
    PopFromScope(),              // pop last pushed object from scope chain
    FnCall(String, Operand),     // Operand = (...) with another operands inside (like a JS object)
    Call(Operand, Operand),      // Call the first operand as a function
    ForInStart(String, Operand), // create an iterator on an object
    ForInNext(String, Operand),  // get the next key of the object
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

// Store a label to jump at when we encouter a continue or a break
pub struct LoopContext {
    continue_label: i64,
    break_label: i64,
}

// Store a label to jump when we encouter a Return + the var to modify with
// the result of the Return statement (undefined if no expression)
pub struct ReturnContext {
    label: i64,
    variable: &'static String,
}

// AST -> IR
pub struct Compiler {
    pub source: ast::Program,
    pub pos: usize,
    pub output: Program,
    pub label_stack: i64,
    pub loop_stack: Vec<LoopContext>,
    pub return_stack: Vec<ReturnContext>,
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
            ast::Stmt::While { cond, body } => {
                let loop_start = self.new_label();
                let loop_end = self.new_label();

                self.loop_stack.push(LoopContext {
                    continue_label: loop_start,
                    break_label: loop_end,
                });

                let cond_val = self.compile_expr(cond);
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::JumpIf(cond_val.clone(), loop_start),
                });
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(loop_end),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(loop_start),
                });

                self.compile_stmt(*body);

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(loop_start),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(loop_end),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::JumpIf(cond_val, loop_start),
                });

                self.loop_stack.pop();
            }
            ast::Stmt::For {
                init,
                cond,
                update,
                body,
            } => {
                // you must read section 12.6.2 of the ES1 spec
                // to understand what's happening here

                if let Some(i) = init {
                    self.compile_for_init(i);
                }

                let loop_start = self.new_label();
                let loop_end = self.new_label();

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(loop_start),
                });

                if let Some(c) = cond {
                    let cond_val = self.compile_expr(c);
                    let body_label = self.new_label();

                    self.output.body.push(Instruction::Call {
                        function: SoloFunction::JumpIf(cond_val, body_label),
                    });
                    self.output.body.push(Instruction::Call {
                        function: SoloFunction::Jump(loop_end),
                    });
                    self.output.body.push(Instruction::Call {
                        function: SoloFunction::Label(body_label),
                    });
                }

                self.compile_stmt(*body); // break, continue and return are handled in there

                if let Some(u) = update {
                    self.compile_expr(u); // unused result to trigger side effects
                }

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(loop_start),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(loop_end),
                });
            }
            ast::Stmt::ForIn { var, expr, body } => {
                let obj_val = self.compile_expr(expr);

                let id = self.new_label();
                let iter_var = format!("__fi_iter_{}", id);
                let key_var = format!("__fi_key_{}", id);
                let done_var = format!("__fi_done_{}", id);

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::ForInStart(iter_var.clone(), obj_val),
                });

                let loop_start = self.new_label();
                let loop_end = self.new_label();

                self.loop_stack.push(LoopContext {
                    continue_label: loop_start,
                    break_label: loop_end,
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(loop_start),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::ForInNext(
                        key_var.clone(),
                        Operand::Var(iter_var.clone()),
                    ),
                });

                self.output.body.push(Instruction::Classic {
                    dest: done_var.clone(),
                    function: Function::Equal(
                        Operand::Var(key_var.clone()),
                        Operand::Const(Const::Undefined),
                    ),
                });
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::JumpIf(Operand::Var(done_var.clone()), loop_end),
                });

                self.output.body.push(Instruction::Assign {
                    dest: var,
                    src: Operand::Var(key_var.clone()),
                });

                self.compile_stmt(*body);

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(loop_start),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(loop_end),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Kill(Operand::Var(iter_var)),
                });
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Kill(Operand::Var(key_var)),
                });
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Kill(Operand::Var(done_var)),
                });
            }
            ast::Stmt::Continue => match self.loop_stack.last() {
                Some(ctx) => {
                    self.output.body.push(Instruction::Call {
                        function: SoloFunction::Jump(ctx.continue_label),
                    });
                }
                None => self.error("Continue statement outside of loop"),
            },
            ast::Stmt::Break => match self.loop_stack.last() {
                Some(ctx) => {
                    self.output.body.push(Instruction::Call {
                        function: SoloFunction::Jump(ctx.break_label),
                    });
                }
                None => self.error("Break statement outside of loop"),
            },
            ast::Stmt::Return(xpr) => {
                let mut expr_val = Operand::Const(Const::Undefined);

                if let Some(expr) = xpr {
                    expr_val = self.compile_expr(expr);
                }

                match self.return_stack.last() {
                    Some(ctx) => {
                        self.output.body.push(Instruction::Classic {
                            dest: ctx.variable.clone(),
                            function: Function::Noop(expr_val),
                        });
                        self.output.body.push(Instruction::Call {
                            function: SoloFunction::Jump(ctx.label),
                        });
                    }
                    None => self.error("Return statement outside of function"),
                }
            }
            // I hate this thing
            ast::Stmt::With { expr, body } => {
                let expr_val = self.compile_expr(expr);

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::PushToScope(expr_val),
                });

                self.compile_stmt(*body);

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::PopFromScope(),
                });
            }

            ast::Stmt::Empty => {}
            _ => self.error("Unsupported statement type"),
        }
    }

    fn compile_expr(&mut self, _expr: ast::Expr) -> Operand {
        return Operand::Const(Const::Number(0.0));
    }

    fn compile_for_init(&mut self, _forinit: ast::ForInit) -> Operand {
        return Operand::Const(Const::Number(0.0));
    }
}
