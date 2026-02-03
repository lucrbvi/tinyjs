/*
 * TinyJS's IR is a simple Three-Adress Code-ish. Here are cool features:
 *  - 'Label' as a function: easier to optimize jumps and constant folding
 *  - Arrays are dynamic and are used to represent objects and arrays (you can delete a key by setting it to Undefined)
 *  - Prototypes properties are injected in an object by the AST->IR compiler
 *
 * Here is an example of a program in our IR
 * JS version: 'function add(a, b) { return a+b }; add(5, 2)'
 * IR version:
 *  FnDeclare("add", 2)
 *      Return(Add(arg[0], arg[1]))
 *  FnEnd()
 *  FnCall([5, 2])
 */

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
    Add((Operand, Operand)),
    Sub((Operand, Operand)),
    Mul((Operand, Operand)),
    Mod((Operand, Operand)),
    Div((Operand, Operand)),
    Pow((Operand, Operand)),
    Inv(Operand), // return the inverse of a boolean (true -> false ; false -> true) 
    Equal((Operand, Operand)), // a == b
    NotEqual((Operand, Operand)), // a != b
    LessThan((Operand, Operand)) // a < b
    GreaterThan((Operand, Operand)) // a > b
    LessThanEqual((Operand, Operand)) // a <= b
    GreaterThanEqual((Operand, Operand)) // a >= b
}

// Functions that do not return anything
#[derive(Debug)]
pub enum SoloFunction {
    Label(i64), // create a "target" for jumps ; the "key" is it's only argument, must be a Number
    JumpIf((Operand, i64)), // jump to a label if the argument is true, only accept Boolean
    Jump(i64),
    Kill(Operand), // dereference a var
    FnStart((String, i64)), // declare the start of a function block (string = name, i64 = number
                            // of arguments)
    FnEnd(), // end a function block
    Return(Option<Operand>),
    FnCall(String, Operand), // Operand = (...) with another operands inside (like a JS object)
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
