use super::Value;

#[derive(Debug)]
pub enum Instruction {
    // TODO: GML runtime instructions
    InterpretationError { error: String },
}

/// Node representing one value in an expression.
pub enum Node {
    Literal {
        value: Value,
    },
    Function {
        args: Box<[Node]>,
        function: fn(&[Value]) -> Value,
    },
    Script {
        args: Box<[Node]>,
        script_id: usize,
    },
    Field {
        index: usize,
        array: Option<ArrayAccessor>,
        owner: Box<Node>,
        value: Box<Node>,
    },
    Variable, // TODO - need an instance variable enum
    GlobalVariable, // TODO - need a global variable enum (is there even a list of these anywhere?)
    Binary {
        left: Box<Node>,
        right: Box<Node>,
        operator: fn(&Value, &Value) -> Value,
    },
    Unary {
        child: Box<Node>,
        operator: fn(&Value) -> Value,
    }

}

/// Represents an array accessor, which can be either 1D or 2D.
/// Variables with 0D arrays, and ones with no array accessor, implicitly refer to [0].
/// Anything beyond a 2D array results in a runtime error.
pub enum ArrayAccessor {
    Single(Box<Node>),
    Double(Box<Node>, Box<Node>),
}

pub struct Error {
    pub reason: String,
    // Probably could add more useful things later.
}
