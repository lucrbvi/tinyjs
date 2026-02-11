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

use std::process::exit;

#[derive(Debug, Clone)]
pub enum Operand {
    Var(String), // reference to another variable
    Const(Const), // initial value
}

pub struct Program {
    pub body: Vec<Instruction>
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
    Kill(Operand), // dereference a var
    FnStart(String, i64), // declare the start of a function block (string = name, i64 = number
                            // of arguments)
    FnEnd(), // end a function block
    Return(Option<Operand>),
    PushToScope(Operand), // push an object in scope chain (for `with`)
    RemoveFromScope(), // pop last pushed object from scope chain
    FnCall(String, Operand), // Operand = (...) with another operands inside (like a JS object)
    Call(Operand, Operand) // Call the first operand as a function
}

#[derive(Debug)]
pub enum Instruction {
    Assign { // a = b
        dest: String,
        src: Operand,
    },
    Call { // label(1) ; jumpif(1==2, 1)
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
    fn advance(&mut self) {
        self.pos += 1;
    }

    fn peek(&self) -> &ast::Stmt {
        &self.source.body[self.pos]
    }

    fn emit(&mut self, instr: Instruction) {
        self.output.body.push(instr);
    }

    fn error(&mut self, msg: String) {
        println!("IR Compiler error: {}", msg);
        exit(-1);
    }

    fn new_label(&mut self) -> Instruction {
        let id = self.new_label_id();
        Instruction::Call {
            function: SoloFunction::Label(id),
        }
    }

    fn new_label_id(&mut self) -> i64 {
        self.label_stack += 1;
        self.label_stack
    }

    // big switch statement
    pub fn parse(&mut self) {
        let stmt = self.peek();
        match stmt {
            ast::Stmt::Var(_) => {
                self.parse_var();
            },
            ast::Stmt::Function(_) => {
                self.parse_function();
            },
            ast::Stmt::Block(_) => {
                self.parse_block();
            },
            ast::Stmt::Expr(_) => {
                self.parse_expression();
            },
            ast::Stmt::If { cond: _, then_: _, else_: _ } => {
                self.parse_if(stmt);
            },
            ast::Stmt::While { cond: _, body: _ } => {
                self.parse_while(stmt);
            },
            ast::Stmt::ForIn { var: _, expr: _, body: _ } => {
                self.parse_for_in();
            },
            ast::Stmt::With { expr: _, body: _ } => {
                self.parse_with(stmt);
            },
            _ => {
                self.advance();
                self.parse();
            }
        }
    }

    /*
     * JS version: while(true) { console.log("hi") }
     *
     * IR version:
     *  label(0)
     *      jumpif(cond, 1)
     *      jump(2)
     *  label(1)
     *      call(console['log'], ("hi"))
     *  label(2)
     */
    pub fn parse_while(&mut self, s: &ast::Stmt) {
        if s != &(ast::Stmt::While { cond: _, body: _ }) {
            self.error(format!("expected a while statement in parse_while but got {:#?}", s));
        }

        let begin_id = self.new_label_id();
        let body_id = self.new_label_id();
        let exit_id = self.new_label_id();

<<<<<<< HEAD
        self.emit(begin);
        let cond = self.parse_expression(s.cond);
        self.emit(Instruction::Call{
            function: SoloFunction::JumpIf(cond, self.label_stack), // jump to body if cond == true
        });
        self.emit(Instruction::Call{
            function: SoloFunction::Jump(self.label_stack + 1), // jump to exit
=======
        self.emit(Instruction::Call {
            function: SoloFunction::Label(begin_id),
        });
        let cond = self.parse_expression(s.cond);
        self.emit(Instruction::Call {
            function: SoloFunction::JumpIf((cond, body_id)),
>>>>>>> 9694962 (ir design is done)
        });
        self.emit(Instruction::Call {
            function: SoloFunction::Jump(exit_id),
        });
        self.emit(Instruction::Call {
            function: SoloFunction::Label(body_id),
        });
        let body = self.parse_statement();
        self.emit(body);
        self.emit(Instruction::Call {
            function: SoloFunction::Jump(begin_id),
        });
        self.emit(Instruction::Call {
            function: SoloFunction::Label(exit_id),
        });
    }

    /*
     *  label(0)
     *   JumpIf(1 == 2, 1)
     *   Jump(2)
     *  label(1) // body
     *   ....
     *  label(2) // else
     *   ....
     *  label(3) // exit
     */
    pub fn parse_if(&mut self, s: &ast::Stmt) {
        if s != &(ast::Stmt::If { cond: _, then_: _, else_: _ }) {
            self.error(format!("expected a if statement in parse_if but got {:#?}", s));
        }

        let begin_id = self.new_label_id();
        let body_id = self.new_label_id();
        let else_id = self.new_label_id();
        let exit_id = self.new_label_id();

        self.emit(Instruction::Call {
            function: SoloFunction::Label(begin_id),
        });
        let cond = self.parse_expression(s.cond);
        self.emit(Instruction::Call {
            function: SoloFunction::JumpIf((cond, body_id)),
        });
        self.emit(Instruction::Call {
            function: SoloFunction::Jump(else_id),
        });

        self.emit(Instruction::Call {
            function: SoloFunction::Label(body_id),
        });
        let body = self.parse_statement();
        self.emit(body);
        self.emit(Instruction::Call {
            function: SoloFunction::Jump(exit_id),
        });

        self.emit(Instruction::Call {
            function: SoloFunction::Label(else_id),
        });
        let else_body = self.parse_statement();
        self.emit(else_body);

        self.emit(Instruction::Call {
            function: SoloFunction::Label(exit_id),
        });
    }

    pub fn parse_with(&mut self, s: &ast::Stmt) {
        if s != &(ast::Stmt::With { expr: _, body: _ }) {
            self.error(format!("expected a with statement in parse_with but got {:#?}", s));
        }

        let scope_obj = match s {
            ast::Stmt::With { expr, body: _ } => self.parse_expression(expr),
            _ => unreachable!(),
        };

        self.emit(Instruction::Call {
            function: SoloFunction::PushToScope(scope_obj),
        });

        let body = self.parse_statement();
        self.emit(body);

        self.emit(Instruction::Call {
            function: SoloFunction::RemoveFromScope(),
        });
    }
}
