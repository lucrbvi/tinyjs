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
        error::fail(format!("AST->IR Compiler error: {}", message));
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
                for (name, init) in v {
                    match init {
                        Some(ast::Expr::Literal(ast::Literal::Object(props))) => {
                            for (key, value) in props {
                                let prop_name = match key {
                                    ast::PropertyKey::Identifier(s) => s,
                                    ast::PropertyKey::String(s) => s,
                                    ast::PropertyKey::Number(n) => format!("{}", n),
                                };
                                let val = self.compile_expr(value);
                                self.output.body.push(Instruction::Assign {
                                    dest: format!("{}_{}", name, prop_name),
                                    src: val,
                                });
                            }
                            self.output.body.push(Instruction::Assign {
                                dest: name,
                                src: Operand::Const(Const::Undefined),
                            });
                        }
                        Some(ast::Expr::Literal(ast::Literal::Array(elements))) => {
                            for (idx, elem) in elements.into_iter().enumerate() {
                                let val = self.compile_expr(elem);
                                self.output.body.push(Instruction::Assign {
                                    dest: format!("{}_{}", name, idx),
                                    src: val,
                                });
                            }
                            self.output.body.push(Instruction::Assign {
                                dest: name,
                                src: Operand::Const(Const::Undefined),
                            });
                        }
                        Some(expr) => {
                            let e = self.compile_expr(expr);
                            self.output
                                .body
                                .push(Instruction::Assign { dest: name, src: e });
                        }
                        None => {
                            self.output.body.push(Instruction::Assign {
                                dest: name,
                                src: Operand::Const(Const::Undefined),
                            });
                        }
                    }
                }
            }
            ast::Stmt::Expr(e) => {
                self.compile_expr(e);
            }
            ast::Stmt::Function(func) => {
                if let Some(name) = func.name {
                    let ret_label = self.new_label();
                    let ret_var: &'static String = Box::leak(Box::new(format!("__ret_{}", name)));

                    self.return_stack.push(ReturnContext {
                        label: ret_label,
                        variable: ret_var,
                    });

                    self.output.body.push(Instruction::Call {
                        function: SoloFunction::FnStart(name, func.params.len() as i64),
                    });

                    for stmt in func.body {
                        self.compile_stmt(stmt);
                    }

                    self.output.body.push(Instruction::Call {
                        function: SoloFunction::Label(ret_label),
                    });
                    self.output.body.push(Instruction::Call {
                        function: SoloFunction::FnEnd(),
                    });

                    self.return_stack.pop();
                } else {
                    self.error("Function statement must have a name");
                }
            }
            ast::Stmt::If { cond, then_, else_ } => {
                let cond_label = self.new_label();
                let then_label = self.new_label();
                let else_label = self.new_label();
                let end_label = self.new_label();

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(cond_label),
                });
                let condop = self.compile_expr(cond);
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::JumpIf(condop, then_label),
                });
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(else_label),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(then_label),
                });
                self.compile_stmt(*then_);
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(end_label),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(else_label),
                });
                if let Some(els) = else_ {
                    self.compile_stmt(*els);
                }
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(end_label),
                });
            }
            ast::Stmt::While { cond, body } => {
                let cond_label = self.new_label();
                let loop_start = self.new_label();
                let loop_end = self.new_label();

                self.loop_stack.push(LoopContext {
                    continue_label: cond_label,
                    break_label: loop_end,
                });

                // Évaluation de la condition
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(cond_label),
                });
                let cond_val = self.compile_expr(cond);
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::JumpIf(cond_val, loop_start),
                });
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(loop_end),
                });

                // Corps de la boucle
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(loop_start),
                });

                self.compile_stmt(*body);

                // Retour à l'évaluation de la condition
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(cond_label),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(loop_end),
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

                let for_label = self.new_label();
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(for_label),
                });

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
                let expr_val = match xpr {
                    Some(expr) => self.compile_expr(expr),
                    None => Operand::Const(Const::Undefined),
                };

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Return(Some(expr_val.clone())),
                });

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
        }
    }

    fn compile_expr(&mut self, expr: ast::Expr) -> Operand {
        match expr {
            ast::Expr::Identifier(name) => Operand::Var(name),
            ast::Expr::Literal(lit) => match lit {
                ast::Literal::Number(n) => Operand::Const(Const::Number(n)),
                ast::Literal::String(s) => Operand::Const(Const::String(s)),
                ast::Literal::Bool(b) => Operand::Const(Const::Boolean(b)),
                ast::Literal::Null => Operand::Const(Const::Null),
                ast::Literal::Undefined => Operand::Const(Const::Undefined),
                ast::Literal::Array(elements) => {
                    let array_var = format!("__t{}", self.new_label());
                    for (idx, elem) in elements.into_iter().enumerate() {
                        let val = self.compile_expr(elem);
                        self.output.body.push(Instruction::Assign {
                            dest: format!("{}_{}", array_var, idx),
                            src: val,
                        });
                    }
                    self.output.body.push(Instruction::Assign {
                        dest: array_var.clone(),
                        src: Operand::Const(Const::Undefined),
                    });
                    Operand::Var(array_var)
                }
                ast::Literal::Object(props) => {
                    let obj_var = format!("__t{}", self.new_label());
                    for (key, value) in props {
                        let prop_name = match key {
                            ast::PropertyKey::Identifier(s) => s,
                            ast::PropertyKey::String(s) => s,
                            ast::PropertyKey::Number(n) => format!("{}", n),
                        };
                        let val = self.compile_expr(value);
                        self.output.body.push(Instruction::Assign {
                            dest: format!("{}_{}", obj_var, prop_name),
                            src: val,
                        });
                    }
                    // Un objet (même vide) doit être truthy pour que !{} == false
                    // On utilise true comme marqueur d'objet existant
                    self.output.body.push(Instruction::Assign {
                        dest: obj_var.clone(),
                        src: Operand::Const(Const::Boolean(true)),
                    });
                    Operand::Var(obj_var)
                }
            },
            ast::Expr::Binary { op, left, right } => {
                let l = self.compile_expr(*left);
                let r = self.compile_expr(*right);
                let dest = format!("__t{}", self.new_label());
                let f = match op {
                    ast::BinOp::Add => Function::Add(l, r),
                    ast::BinOp::Sub => Function::Sub(l, r),
                    ast::BinOp::Mul => Function::Mul(l, r),
                    ast::BinOp::Div => Function::Div(l, r),
                    ast::BinOp::Mod => Function::Mod(l, r),
                    ast::BinOp::Eq => Function::Equal(l, r),
                    ast::BinOp::Ne => Function::NotEqual(l, r),
                    ast::BinOp::Lt => Function::LessThan(l, r),
                    ast::BinOp::Gt => Function::GreaterThan(l, r),
                    ast::BinOp::Le => Function::LessThanEqual(l, r),
                    ast::BinOp::Ge => Function::GreaterThanEqual(l, r),
                    _ => self.error("unsupported binary op"),
                };
                self.output.body.push(Instruction::Classic {
                    dest: dest.clone(),
                    function: f,
                });
                Operand::Var(dest)
            }
            ast::Expr::Unary { op, expr } => {
                match op {
                    ast::UnaryOp::Pos => self.compile_expr(*expr),
                    ast::UnaryOp::Neg => {
                        let e = self.compile_expr(*expr);
                        let dest = format!("__t{}", self.new_label());
                        self.output.body.push(Instruction::Classic {
                            dest: dest.clone(),
                            function: Function::Sub(Operand::Const(Const::Number(0.0)), e),
                        });
                        Operand::Var(dest)
                    }
                    ast::UnaryOp::Not => {
                        let e = self.compile_expr(*expr);
                        let dest = format!("__t{}", self.new_label());
                        self.output.body.push(Instruction::Classic {
                            dest: dest.clone(),
                            function: Function::Inv(e),
                        });
                        Operand::Var(dest)
                    }
                    ast::UnaryOp::Delete => {
                        // delete obj.a -> set obj_a = undefined, return true
                        if let ast::Expr::Member { object, property } = expr.as_ref() {
                            if let ast::Expr::Identifier(obj_name) = object.as_ref() {
                                let dest = format!("{}_{}", obj_name, property);
                                self.output.body.push(Instruction::Assign {
                                    dest,
                                    src: Operand::Const(Const::Undefined),
                                });
                                return Operand::Const(Const::Boolean(true));
                            }
                        }
                        self.error("unsupported delete target")
                    }
                    _ => self.error("unsupported unary op"),
                }
            }
            ast::Expr::Assign { target, op, value } => {
                let val = self.compile_expr(*value);
                match *target {
                    ast::Expr::Identifier(name) => {
                        match op {
                            ast::AssignOp::Assign => {
                                self.output.body.push(Instruction::Assign {
                                    dest: name.clone(),
                                    src: val,
                                });
                            }
                            ast::AssignOp::AddAssign => {
                                self.output.body.push(Instruction::Classic {
                                    dest: name.clone(),
                                    function: Function::Add(Operand::Var(name.clone()), val),
                                });
                            }
                            ast::AssignOp::SubAssign => {
                                self.output.body.push(Instruction::Classic {
                                    dest: name.clone(),
                                    function: Function::Sub(Operand::Var(name.clone()), val),
                                });
                            }
                            _ => self.error("unsupported assign op"),
                        };
                        Operand::Var(name)
                    }
                    ast::Expr::Member { object, property } => {
                        let dest = match *object {
                            ast::Expr::Identifier(obj_name) => format!("{}_{}", obj_name, property),
                            ast::Expr::Member {
                                object: inner,
                                property: inner_prop,
                            } => {
                                if let ast::Expr::Identifier(inner_name) = *inner {
                                    format!("{}_{}_{}", inner_name, inner_prop, property)
                                } else {
                                    self.error("unsupported assign target")
                                }
                            }
                            _ => self.error("unsupported assign target"),
                        };
                        self.output.body.push(Instruction::Assign {
                            dest: dest.clone(),
                            src: val,
                        });
                        Operand::Var(dest)
                    }
                    ast::Expr::Index { object, index } => {
                        if let ast::Expr::Identifier(obj_name) = *object {
                            if let ast::Expr::Identifier(idx_name) = *index {
                                let dest = format!("{}_{}", obj_name, idx_name);
                                self.output.body.push(Instruction::Assign {
                                    dest: dest.clone(),
                                    src: val,
                                });
                                Operand::Var(dest)
                            } else {
                                self.error("unsupported index target")
                            }
                        } else {
                            self.error("unsupported index target")
                        }
                    }
                    _ => self.error("unsupported assign target"),
                }
            }
            ast::Expr::Member { object, property } => match *object {
                ast::Expr::Identifier(obj_name) => {
                    Operand::Var(format!("{}_{}", obj_name, property))
                }
                ast::Expr::Member {
                    object: inner,
                    property: inner_prop,
                } => {
                    if let ast::Expr::Identifier(inner_name) = *inner {
                        Operand::Var(format!("{}_{}_{}", inner_name, inner_prop, property))
                    } else {
                        self.error("unsupported nested member")
                    }
                }
                _ => self.error("unsupported member"),
            },
            ast::Expr::Call { callee, args } => {
                let func_name = match *callee {
                    ast::Expr::Identifier(name) => name,
                    ast::Expr::Member { object, property } => {
                        if let ast::Expr::Identifier(obj_name) = *object {
                            format!("{}_{}", obj_name, property)
                        } else {
                            self.error("complex callee")
                        }
                    }
                    _ => self.error("unsupported callee"),
                };
                let args_op = self.compile_expr(*args);
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::FnCall(func_name, args_op),
                });
                Operand::Const(Const::Undefined)
            }
            ast::Expr::Ternary { cond, then_, else_ } => {
                let dest = format!("__t{}", self.new_label());
                let else_label = self.new_label();
                let end_label = self.new_label();

                let c = self.compile_expr(*cond);
                let not_c = format!("__t{}", self.new_label());
                self.output.body.push(Instruction::Classic {
                    dest: not_c.clone(),
                    function: Function::Inv(c),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::JumpIf(Operand::Var(not_c), else_label),
                });

                let t = self.compile_expr(*then_);
                self.output.body.push(Instruction::Assign {
                    dest: dest.clone(),
                    src: t,
                });
                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Jump(end_label),
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(else_label),
                });
                let e = self.compile_expr(*else_);
                self.output.body.push(Instruction::Assign {
                    dest: dest.clone(),
                    src: e,
                });

                self.output.body.push(Instruction::Call {
                    function: SoloFunction::Label(end_label),
                });

                Operand::Var(dest)
            }
            ast::Expr::Update {
                op,
                prefix,
                argument,
            } => {
                if let ast::Expr::Identifier(name) = *argument {
                    let arg = Operand::Var(name.clone());
                    let one = Operand::Const(Const::Number(1.0));

                    if prefix {
                        // Pré-incrément: ++i
                        let func = match op {
                            ast::UpdateOp::Inc => Function::Add(arg.clone(), one),
                            ast::UpdateOp::Dec => Function::Sub(arg.clone(), one),
                        };
                        self.output.body.push(Instruction::Classic {
                            dest: name.clone(),
                            function: func,
                        });
                        Operand::Var(name)
                    } else {
                        // Post-incrément: i++
                        // Sauvegarder la valeur d'origine
                        let old_val = format!("__t{}", self.new_label());
                        self.output.body.push(Instruction::Classic {
                            dest: old_val.clone(),
                            function: Function::Noop(arg.clone()),
                        });
                        // Incrémenter la variable
                        let func = match op {
                            ast::UpdateOp::Inc => Function::Add(arg.clone(), one),
                            ast::UpdateOp::Dec => Function::Sub(arg.clone(), one),
                        };
                        self.output.body.push(Instruction::Classic {
                            dest: name.clone(),
                            function: func,
                        });
                        Operand::Var(old_val)
                    }
                } else {
                    self.error("update on non-identifier")
                }
            }
            ast::Expr::Sequence(exprs) => {
                let mut last = Operand::Const(Const::Undefined);
                for e in exprs {
                    last = self.compile_expr(e);
                }
                last
            }
            _ => self.error("unsupported expr"),
        }
    }

    fn compile_for_init(&mut self, forinit: ast::ForInit) -> Operand {
        match forinit {
            ast::ForInit::Var(vars) => {
                for (name, init) in vars {
                    let val = match init {
                        Some(expr) => self.compile_expr(expr),
                        None => Operand::Const(Const::Undefined),
                    };
                    self.output.body.push(Instruction::Assign {
                        dest: name,
                        src: val,
                    });
                }
                Operand::Const(Const::Undefined)
            }
            ast::ForInit::Expr(expr) => self.compile_expr(expr),
        }
    }
}
