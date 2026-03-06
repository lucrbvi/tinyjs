use crate::ir::{Const, Function, Instruction, Operand, Program, SoloFunction};

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    /// Push a constant onto the stack
    Const(Const),
    /// Load a variable onto the stack
    Load(String),
    /// Store top of stack into variable (pops the value)
    Store(String),
    /// Pop and discard top of stack
    Pop,
    /// Duplicate top of stack
    Dup,
    /// Swap top two stack elements
    Swap,

    // Arithmetic operations (pop 2, push 1)
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Comparison operations (pop 2, push 1 bool)
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,

    // Unary operations (pop 1, push 1)
    Neg, // arithmetic negation
    Not, // logical negation

    // Control flow (labels are resolved to offsets during compilation)
    /// Unconditional jump to offset
    Jump(i32),
    /// Jump to offset if top of stack is truthy (pops the condition)
    JumpIf(i32),
    /// Jump to offset if top of stack is falsy (pops the condition)
    JumpIfNot(i32),
    // Function operations
    /// Define function start: name, arg_count, body_offset
    FnStart {
        name: String,
        argc: u8,
        body_offset: i32,
    },
    /// End of function definition
    FnEnd,
    /// Call function by name with N arguments (args are on stack)
    Call {
        name: String,
        argc: u8,
    },
    /// Call top of stack as a function (argc args on stack below)
    CallDynamic {
        argc: u8,
    },
    /// Return from function (optionally with value from stack)
    Return {
        has_value: bool,
    },

    // Scope operations
    /// Push value from stack to scope chain
    PushScope,
    /// Pop from scope chain
    PopScope,

    // Object/Array operations
    /// Delete variable
    Kill(String),
    /// Create object from N key-value pairs (2*N values on stack)
    MakeObject {
        pairs: u16,
    },
    /// Create array from N values (N values on stack)
    MakeArray {
        len: u16,
    },
    /// Get property: obj[prop] (obj and prop on stack, pushes value)
    GetProp,
    /// Set property: obj[prop] = val (obj, prop, val on stack)
    SetProp,
    /// Check if property exists (obj and key on stack, pushes bool)
    HasProp,

    // Iterator operations for for..in
    /// Start iteration, pushes iterator onto stack
    ForInStart,
    /// Get next key from iterator (iterator on stack), pushes key or undefined
    ForInNext,

    Nop,
    Halt,
}

/// A compiled bytecode program
#[derive(Debug)]
pub struct Bytecode {
    pub instructions: Vec<OpCode>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    pub fn emit(&mut self, op: OpCode) {
        self.instructions.push(op);
    }

    pub fn pos(&self) -> usize {
        self.instructions.len()
    }

    /// Patch a jump instruction at position with target offset
    pub fn patch_jump(&mut self, pos: usize, target: i32) {
        match self.instructions[pos] {
            OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_) => {
                self.instructions[pos] = match self.instructions[pos] {
                    OpCode::Jump(_) => OpCode::Jump(target),
                    OpCode::JumpIf(_) => OpCode::JumpIf(target),
                    OpCode::JumpIfNot(_) => OpCode::JumpIfNot(target),
                    _ => unreachable!(),
                };
            }
            _ => panic!("Cannot patch non-jump instruction"),
        }
    }
}

pub fn compile_to_bytecode(ir: Program) -> Bytecode {
    let mut compiler = BytecodeCompiler::new();
    compiler.compile(ir);
    compiler.bytecode
}

/// IR to Bytecode compiler
struct BytecodeCompiler {
    bytecode: Bytecode,
    /// Map IR labels to bytecode positions
    label_map: std::collections::HashMap<i64, usize>,
    /// Positions of jump instructions that need backpatching (label_id)
    pending_jumps: Vec<(usize, i64)>,
    /// Pending function body jumps: (position_of_jump, label_to_emit_at_end)
    pending_fn_jumps: Vec<(usize, i64)>,
    /// Function entry points: name -> bytecode position
    function_entries: std::collections::HashMap<String, usize>,
    /// Counter for generating unique end-of-function labels
    label_counter: i64,
    /// Stack of function end labels (for nested functions)
    fn_end_label_stack: Vec<i64>,
    /// Map function end label -> FnStart position (for backpatching body_offset)
    fn_start_positions: std::collections::HashMap<i64, usize>,
}

impl BytecodeCompiler {
    fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            label_map: std::collections::HashMap::new(),
            pending_jumps: Vec::new(),
            pending_fn_jumps: Vec::new(),
            function_entries: std::collections::HashMap::new(),
            label_counter: 10000, // Start high to avoid conflicts with IR labels
            fn_end_label_stack: Vec::new(),
            fn_start_positions: std::collections::HashMap::new(),
        }
    }

    fn new_label(&mut self) -> i64 {
        self.label_counter += 1;
        self.label_counter
    }

    fn compile(&mut self, ir: Program) {
        // First pass: collect label positions and emit bytecode
        for instr in ir.body {
            self.compile_instruction(instr);
        }

        // Second pass: backpatch jumps
        self.resolve_jumps();

        self.bytecode.emit(OpCode::Halt);
    }

    fn compile_instruction(&mut self, instr: Instruction) {
        match instr {
            Instruction::Assign { dest, src } => {
                self.emit_operand(src);
                self.bytecode.emit(OpCode::Store(dest));
            }
            Instruction::Call { function } => {
                self.compile_solo_function(function);
            }
            Instruction::Classic { dest, function } => {
                self.compile_function(function);
                self.bytecode.emit(OpCode::Store(dest));
            }
        }
    }

    fn emit_operand(&mut self, op: Operand) {
        match op {
            Operand::Var(name) => {
                self.bytecode.emit(OpCode::Load(name));
            }
            Operand::Const(c) => {
                self.bytecode.emit(OpCode::Const(c));
            }
        }
    }

    fn compile_function(&mut self, func: Function) {
        use Function::*;
        match func {
            Noop(op) => {
                self.emit_operand(op);
            }
            Add(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Add);
            }
            Sub(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Sub);
            }
            Mul(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Mul);
            }
            Div(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Div);
            }
            Mod(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Mod);
            }
            Pow(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Pow);
            }
            Inv(a) => {
                self.emit_operand(a);
                self.bytecode.emit(OpCode::Not);
            }
            Equal(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Eq);
            }
            NotEqual(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Ne);
            }
            LessThan(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Lt);
            }
            GreaterThan(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Gt);
            }
            LessThanEqual(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Le);
            }
            GreaterThanEqual(a, b) => {
                self.emit_operand(a);
                self.emit_operand(b);
                self.bytecode.emit(OpCode::Ge);
            }
        }
    }

    fn compile_solo_function(&mut self, func: SoloFunction) {
        use SoloFunction::*;
        match func {
            Label(id) => {
                let pos = self.bytecode.pos();
                self.label_map.insert(id, pos);
            }
            JumpIf(cond, label) => {
                self.emit_operand(cond);
                let jump_pos = self.bytecode.pos();
                self.bytecode.emit(OpCode::JumpIf(0)); // Will be backpatched
                self.pending_jumps.push((jump_pos, label));
            }
            Jump(label) => {
                let jump_pos = self.bytecode.pos();
                self.bytecode.emit(OpCode::Jump(0)); // Will be backpatched
                self.pending_jumps.push((jump_pos, label));
            }
            Kill(op) => {
                if let Operand::Var(name) = op {
                    self.bytecode.emit(OpCode::Kill(name));
                }
            }
            FnStart(name, argc) => {
                let end_label = self.new_label();
                self.fn_end_label_stack.push(end_label);

                let jump_pos = self.bytecode.pos();
                self.bytecode.emit(OpCode::Jump(0)); // Will be backpatched
                self.pending_fn_jumps.push((jump_pos, end_label));

                let fn_start_pos = self.bytecode.pos();
                self.bytecode.emit(OpCode::FnStart {
                    name: name.clone(),
                    argc: argc as u8,
                    body_offset: 0, // Will be backpatched
                });
                self.fn_start_positions.insert(end_label, fn_start_pos);

                self.function_entries.insert(name, fn_start_pos + 1);
            }
            FnEnd() => {
                self.bytecode.emit(OpCode::Return { has_value: false });

                let end_label = self
                    .fn_end_label_stack
                    .pop()
                    .expect("FnEnd without matching FnStart");
                let end_pos = self.bytecode.pos();
                self.label_map.insert(end_label, end_pos);
            }
            Return(op) => {
                if let Some(operand) = op {
                    self.emit_operand(operand);
                    self.bytecode.emit(OpCode::Return { has_value: true });
                } else {
                    self.bytecode.emit(OpCode::Return { has_value: false });
                }
            }
            PushToScope(op) => {
                self.emit_operand(op);
                self.bytecode.emit(OpCode::PushScope);
            }
            PopFromScope() => {
                self.bytecode.emit(OpCode::PopScope);
            }
            FnCall(name, args) => {
                self.emit_operand(args); // naive to just push without verifying - can crash the VM
                self.bytecode.emit(OpCode::Call { name, argc: 1 });
            }
            MethodCall(obj, method, args) => {
                self.emit_operand(obj);
                self.bytecode
                    .emit(OpCode::Const(crate::ir::Const::String(method)));
                self.bytecode.emit(OpCode::GetProp);
                self.emit_operand(args);
                self.bytecode.emit(OpCode::CallDynamic { argc: 1 });
            }
            Call(func, args) => {
                self.emit_operand(func);
                self.emit_operand(args);
                self.bytecode.emit(OpCode::CallDynamic { argc: 1 });
            }
            ForInStart(iter_var, obj) => {
                self.emit_operand(obj);
                self.bytecode.emit(OpCode::ForInStart);
                self.bytecode.emit(OpCode::Store(iter_var));
            }
            ForInNext(key_var, iter) => {
                self.emit_operand(iter);
                self.bytecode.emit(OpCode::ForInNext);
                self.bytecode.emit(OpCode::Store(key_var));
            }
            MakeObject(var_name, props) => {
                // Push all key-value pairs onto the stack
                for (key, value) in &props {
                    self.bytecode
                        .emit(OpCode::Const(Const::String(key.clone())));
                    self.emit_operand(value.clone());
                }
                self.bytecode.emit(OpCode::MakeObject {
                    pairs: props.len() as u16,
                });
                self.bytecode.emit(OpCode::Store(var_name));
            }
            MakeArray(var_name, elements) => {
                // Push all elements onto the stack
                for elem in &elements {
                    self.emit_operand(elem.clone());
                }
                self.bytecode.emit(OpCode::MakeArray {
                    len: elements.len() as u16,
                });
                self.bytecode.emit(OpCode::Store(var_name));
            }
        }
    }

    fn resolve_jumps(&mut self) {
        // Resolve pending jumps
        for (pos, label_id) in &self.pending_jumps {
            if let Some(&target) = self.label_map.get(label_id) {
                let offset = target as i32 - *pos as i32;
                self.bytecode.patch_jump(*pos, offset);
            } else {
                panic!("Undefined label: {}", label_id);
            }
        }

        // Resolve function body jumps (skip over function definitions)
        for (pos, end_label) in &self.pending_fn_jumps {
            if let Some(&target) = self.label_map.get(end_label) {
                let offset = target as i32 - *pos as i32;
                self.bytecode.patch_jump(*pos, offset);
            } else {
                panic!("Undefined function end label: {}", end_label);
            }
        }

        // Resolve function body offsets for FnStart instructions
        self.resolve_fn_body_offsets();
    }

    fn resolve_fn_body_offsets(&mut self) {
        for (end_label, fn_start_pos) in &self.fn_start_positions {
            if let Some(&end_pos) = self.label_map.get(end_label) {
                let body_offset = end_pos as i32 - *fn_start_pos as i32;
                // Patch the FnStart instruction at fn_start_pos
                if let OpCode::FnStart {
                    ref name, ref argc, ..
                } = self.bytecode.instructions[*fn_start_pos]
                {
                    let name = name.clone();
                    let argc = *argc;
                    self.bytecode.instructions[*fn_start_pos] = OpCode::FnStart {
                        name,
                        argc,
                        body_offset,
                    };
                } else {
                    panic!("Expected FnStart at position {}", fn_start_pos);
                }
            } else {
                panic!("Undefined function end label: {}", end_label);
            }
        }
    }
}

/// VM Value type
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Undefined,
    Object(Rc<RefCell<std::collections::HashMap<String, Box<Value>>>>),
    Array(Vec<Value>),
    Function { name: String, addr: usize, argc: u8 },
    NativeFunction(fn(&mut VM, &[Value]) -> Result<Value, String>),
}

impl Value {
    fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0 && !n.is_nan(),
            Value::String(s) => !s.is_empty(),
            Value::Null | Value::Undefined => false,
            _ => true, // objects, arrays, functions are truthy
        }
    }

    fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::String(s) => s.parse().unwrap_or(f64::NAN),
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            _ => f64::NAN,
        }
    }

    fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => {
                if n.is_nan() {
                    "NaN".to_string()
                } else if n.is_infinite() {
                    if *n > 0.0 {
                        "Infinity".to_string()
                    } else {
                        "-Infinity".to_string()
                    }
                } else {
                    format!("{}", n)
                }
            }
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Undefined => "undefined".to_string(),
            _ => "[object Object]".to_string(),
        }
    }
}

/// VM execution context
pub struct VM {
    /// Global variables
    globals: std::collections::HashMap<String, Value>,
    /// Local variable stack (for function calls)
    locals: Vec<std::collections::HashMap<String, Value>>,
    /// Operand stack
    stack: Vec<Value>,
    /// Program counter
    pc: usize,
    /// Call stack (return addresses)
    call_stack: Vec<usize>,
    /// Scope chain for with statements
    scope_chain: Vec<Value>,
    /// Bytecode being executed
    bytecode: Bytecode,
    /// Iterator storage for for..in
    iterators: std::collections::HashMap<String, Vec<String>>,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        Self {
            globals: std::collections::HashMap::new(),
            locals: vec![std::collections::HashMap::new()],
            stack: Vec::new(),
            pc: 0,
            call_stack: Vec::new(),
            scope_chain: Vec::new(),
            bytecode,
            iterators: std::collections::HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Value {
        loop {
            let op = self.bytecode.instructions[self.pc].clone();
            self.pc += 1;

            match op {
                OpCode::Const(c) => {
                    let val = self.const_to_value(c);
                    self.stack.push(val);
                }
                OpCode::Load(name) => {
                    let val = self.get_variable(&name);
                    self.stack.push(val);
                }
                OpCode::Store(name) => {
                    let val = self.stack.pop().unwrap();
                    self.set_variable(name, val);
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::Dup => {
                    let val = self.stack.last().unwrap().clone();
                    self.stack.push(val);
                }
                OpCode::Swap => {
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                }
                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = self.add_values(a, b);
                    self.stack.push(result);
                }
                OpCode::Sub => {
                    let b = self.stack.pop().unwrap().to_number();
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Number(a - b));
                }
                OpCode::Mul => {
                    let b = self.stack.pop().unwrap().to_number();
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Number(a * b));
                }
                OpCode::Div => {
                    let b = self.stack.pop().unwrap().to_number();
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Number(a / b));
                }
                OpCode::Mod => {
                    let b = self.stack.pop().unwrap().to_number();
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Number(a % b));
                }
                OpCode::Pow => {
                    let b = self.stack.pop().unwrap().to_number();
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Number(a.powf(b)));
                }
                OpCode::Eq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(self.values_equal(a, b)));
                }
                OpCode::Ne => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(!self.values_equal(a, b)));
                }
                OpCode::Lt => {
                    let b = self.stack.pop().unwrap().to_number();
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Bool(a < b));
                }
                OpCode::Gt => {
                    let b = self.stack.pop().unwrap().to_number();
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Bool(a > b));
                }
                OpCode::Le => {
                    let b = self.stack.pop().unwrap().to_number();
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Bool(a <= b));
                }
                OpCode::Ge => {
                    let b = self.stack.pop().unwrap().to_number();
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Bool(a >= b));
                }
                OpCode::Neg => {
                    let a = self.stack.pop().unwrap().to_number();
                    self.stack.push(Value::Number(-a));
                }
                OpCode::Not => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(!a.is_truthy()));
                }
                OpCode::Jump(offset) => {
                    self.pc = (self.pc as i32 + offset - 1) as usize;
                }
                OpCode::JumpIf(offset) => {
                    let cond = self.stack.pop().unwrap();
                    if cond.is_truthy() {
                        self.pc = (self.pc as i32 + offset - 1) as usize;
                    }
                }
                OpCode::JumpIfNot(offset) => {
                    let cond = self.stack.pop().unwrap();
                    if !cond.is_truthy() {
                        self.pc = (self.pc as i32 + offset - 1) as usize;
                    }
                }
                OpCode::FnStart {
                    name,
                    argc,
                    body_offset,
                } => {
                    _ = body_offset;

                    // Store function definition, skip over body during normal execution
                    let addr = self.pc;
                    // Find FnEnd to skip body
                    let mut depth = 1;
                    let mut end_pos = self.pc;
                    while depth > 0 && end_pos < self.bytecode.instructions.len() {
                        match self.bytecode.instructions[end_pos] {
                            OpCode::FnStart { .. } => depth += 1,
                            OpCode::FnEnd => depth -= 1,
                            _ => {}
                        }
                        end_pos += 1;
                    }
                    // Store function reference
                    let func = Value::Function {
                        name: name.clone(),
                        addr,
                        argc,
                    };
                    self.globals.insert(name, func);
                    // Skip to after FnEnd
                    self.pc = end_pos;
                }
                OpCode::FnEnd => {
                    // Marker, should be handled by FnStart or Return
                }
                OpCode::Call { name, argc } => {
                    let func = self.get_variable(&name);
                    self.call_function(func, argc);
                }
                OpCode::CallDynamic { argc } => {
                    let func = self.stack.pop().unwrap();
                    self.call_function(func, argc);
                }
                OpCode::Return { has_value } => {
                    let val = if has_value {
                        self.stack.pop().unwrap()
                    } else {
                        Value::Undefined
                    };
                    if let Some(ret_addr) = self.call_stack.pop() {
                        self.locals.pop();
                        self.pc = ret_addr;
                        self.stack.push(val);
                    } else {
                        return val;
                    }
                }
                OpCode::PushScope => {
                    let val = self.stack.pop().unwrap();
                    self.scope_chain.push(val);
                }
                OpCode::PopScope => {
                    self.scope_chain.pop();
                }
                OpCode::Kill(name) => {
                    // Remove variable from current scope
                    if let Some(locals) = self.locals.last_mut() {
                        locals.remove(&name);
                    }
                    self.globals.remove(&name);
                }
                OpCode::MakeObject { pairs } => {
                    let mut obj = std::collections::HashMap::new();
                    for _ in 0..pairs {
                        let val = self.stack.pop().unwrap();
                        let key = self.stack.pop().unwrap().to_string();
                        obj.insert(key, Box::new(val));
                    }
                    self.stack.push(Value::Object(Rc::new(RefCell::new(obj))));
                }
                OpCode::MakeArray { len } => {
                    let mut arr = Vec::new();
                    for _ in 0..len {
                        arr.push(self.stack.pop().unwrap());
                    }
                    arr.reverse();
                    self.stack.push(Value::Array(arr));
                }
                OpCode::GetProp => {
                    let key = self.stack.pop().unwrap().to_string();
                    let obj = self.stack.pop().unwrap();
                    let val = self.get_property(obj, &key);
                    self.stack.push(val);
                }
                OpCode::SetProp => {
                    let val = self.stack.pop().unwrap();
                    let key = self.stack.pop().unwrap().to_string();
                    let obj = self.stack.pop().unwrap();
                    self.set_property(obj, key, val);
                }
                OpCode::HasProp => {
                    let key = self.stack.pop().unwrap().to_string();
                    let obj = self.stack.pop().unwrap();
                    let has = self.has_property(&obj, &key);
                    self.stack.push(Value::Bool(has));
                }
                OpCode::ForInStart => {
                    let obj = self.stack.pop().unwrap();
                    let keys = self.get_object_keys(obj);
                    self.stack
                        .push(Value::String(format!("__iter_{}", self.iterators.len())));
                    self.iterators
                        .insert(format!("__iter_{}", self.iterators.len()), keys);
                }
                OpCode::ForInNext => {
                    let iter_key = self.stack.pop().unwrap().to_string();
                    if let Some(keys) = self.iterators.get_mut(&iter_key) {
                        if let Some(key) = keys.pop() {
                            self.stack.push(Value::String(key));
                        } else {
                            self.stack.push(Value::Undefined);
                        }
                    } else {
                        self.stack.push(Value::Undefined);
                    }
                }
                OpCode::Nop => {}
                OpCode::Halt => {
                    return self.stack.pop().unwrap_or(Value::Undefined);
                }
            }
        }
    }

    fn const_to_value(&self, c: Const) -> Value {
        use Const::*;
        match c {
            Number(n) => Value::Number(n),
            String(s) => Value::String(s),
            Boolean(b) => Value::Bool(b),
            Undefined => Value::Undefined,
            Null => Value::Null,
        }
    }

    pub fn get_variable(&self, name: &str) -> Value {
        // Check locals first (innermost scope)
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return val.clone();
            }
        }
        // Check globals
        if let Some(val) = self.globals.get(name) {
            return val.clone();
        }
        // Check scope chain for with statements
        for scope in self.scope_chain.iter().rev() {
            if let Value::Object(obj) = scope {
                if let Some(val) = obj.borrow().get(name) {
                    return val.as_ref().clone();
                }
            }
        }
        Value::Undefined
    }

    fn set_variable(&mut self, name: String, val: Value) {
        // Set in local scope if exists, otherwise create in current local scope
        if let Some(locals) = self.locals.last_mut() {
            locals.insert(name, val);
        } else {
            self.globals.insert(name, val);
        }
    }

    fn add_values(&self, a: Value, b: Value) -> Value {
        // String concatenation if either is string
        match (&a, &b) {
            (Value::String(sa), _) => Value::String(format!("{}{}", sa, b.to_string())),
            (_, Value::String(sb)) => Value::String(format!("{}{}", a.to_string(), sb)),
            _ => Value::Number(a.to_number() + b.to_number()),
        }
    }

    fn values_equal(&self, a: Value, b: Value) -> bool {
        match (&a, &b) {
            (Value::Number(na), Value::Number(nb)) => na == nb,
            (Value::String(sa), Value::String(sb)) => sa == sb,
            (Value::Bool(ba), Value::Bool(bb)) => ba == bb,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Undefined) => true,
            (Value::Undefined, Value::Null) => true,
            _ => false,
        }
    }

    fn call_function(&mut self, func: Value, argc: u8) {
        match func {
            Value::Function {
                name: _,
                addr,
                argc: expected,
            } => {
                if argc != expected {
                    // Type error, but we'll be lenient for now
                }
                // Pop arguments from stack
                let mut args = Vec::new();
                for _ in 0..argc {
                    args.push(self.stack.pop().unwrap());
                }
                args.reverse();

                // Setup call
                self.call_stack.push(self.pc);
                let mut locals = std::collections::HashMap::new();
                // Store args in locals (simplified - would use proper param names)
                for (i, arg) in args.into_iter().enumerate() {
                    locals.insert(format!("arg{}", i), arg);
                }
                self.locals.push(locals);
                self.pc = addr;
            }
            Value::NativeFunction(f) => {
                let mut args = Vec::new();
                for _ in 0..argc {
                    args.push(self.stack.pop().unwrap());
                }
                args.reverse();
                let result = f(self, &args);
                self.stack.push(result.unwrap());
            }
            _ => {
                // Not callable, push undefined
                self.stack.push(Value::Undefined);
            }
        }
    }

    fn get_property(&self, obj: Value, key: &str) -> Value {
        match obj {
            Value::Object(map) => map
                .borrow()
                .get(key)
                .map(|v| v.as_ref().clone())
                .unwrap_or(Value::Undefined),
            Value::Array(arr) => {
                if let Ok(idx) = key.parse::<usize>() {
                    arr.get(idx).cloned().unwrap_or(Value::Undefined)
                } else {
                    Value::Undefined
                }
            }
            _ => Value::Undefined,
        }
    }

    fn set_property(&mut self, obj: Value, key: String, val: Value) {
        if let Value::Object(map) = obj {
            map.borrow_mut().insert(key, Box::new(val));
        }
    }

    fn has_property(&self, obj: &Value, key: &str) -> bool {
        match obj {
            Value::Object(map) => map.borrow().contains_key(key),
            Value::Array(arr) => {
                if let Ok(idx) = key.parse::<usize>() {
                    idx < arr.len()
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn get_object_keys(&self, obj: Value) -> Vec<String> {
        match obj {
            Value::Object(map) => map.borrow().keys().cloned().collect(),
            Value::Array(arr) => (0..arr.len()).map(|i| i.to_string()).collect(),
            _ => Vec::new(),
        }
    }
}
