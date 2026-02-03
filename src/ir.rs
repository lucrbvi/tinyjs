/*
 * TinyJS's IR is a simple SSA one. Here are cool features:
 *  1. 'Label' as a function: easier to optimize jumps and constant folding
 *
 */

pub struct Instruction {
    pub target: Op,
    pub function: Function,
    pub left: Option<Op>,
    pub right: Option<Op>,
}

pub struct Op {
    pub name: String,
    pub type_: Type,
}

// Objects are destructured in multiple variables
// Ex: var a = {b: 15, c: "Hey"}; -> a_b = 15; a_c = "Hey";
pub enum Type {
    Identifier(String),
    String(String),
    Number(f64),
    Boolean(bool),
    Undefined,
    Null,
}

pub enum Function {

}
