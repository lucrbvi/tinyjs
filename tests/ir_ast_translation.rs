// TODO: Update the tests to align on the AST compiler

use tinyjs::ast;
use tinyjs::ir;
use tinyjs::lexer;
use tinyjs::parser;

fn parse_program(source: &str) -> ast::Program {
    let mut lex = lexer::Lexer {
        source: source.to_string(),
        cursor: lexer::Cursor { row: 0, line: 0 },
        line: 0,
        row: 0,
        prev_cr: false,
    };

    let tokens = lex.walk();
    let mut parser = parser::Parser {
        tokens: Vec::new(),
        pos: 0,
        allow_in: true,
        source: source.to_string(),
    };

    parser.parse(tokens)
}

fn compile_ir(source: &str) -> Vec<ir::Instruction> {
    let mut compiler = ir::Compiler {
        source: parse_program(source),
        pos: 0,
        output: ir::Program { body: vec![] },
        label_stack: 0,
        loop_stack: vec![],
        return_stack: vec![],
    };

    compiler.compile();
    compiler.output.body
}

#[test]
fn translates_basic_var_declaration() {
    let out = compile_ir("var a = 1;");

    assert!(matches!(
        out.first(),
        Some(ir::Instruction::Assign {
            dest,
            src: ir::Operand::Const(ir::Const::Number(1.0))
        }) if dest == "a"
    ));
}

#[test]
fn translates_basic_binary_expression() {
    let out = compile_ir("a + 2;");

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Classic {
                function: ir::Function::Add(ir::Operand::Var(left), ir::Operand::Const(ir::Const::Number(2.0))),
                ..
            } if left == "a"
        )
    }));
}

#[test]
fn translates_if_else_control_flow() {
    let out = compile_ir("if (a < 3) b = 1; else b = 2;");

    let label_count = out
        .iter()
        .filter(|instr| {
            matches!(
                instr,
                ir::Instruction::Call {
                    function: ir::SoloFunction::Label(_)
                }
            )
        })
        .count();

    assert!(label_count >= 4);
    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::JumpIf(_, _)
            }
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Const(ir::Const::Number(1.0))
            } if dest == "b"
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Const(ir::Const::Number(2.0))
            } if dest == "b"
        )
    }));
}

#[test]
fn translates_while_loop_control_flow() {
    let out = compile_ir("while (i < 2) i++; ");

    let has_cond_jump = out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::JumpIf(_, _)
            }
        )
    });

    let has_back_jump = out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::Jump(_)
            }
        )
    });

    assert!(has_cond_jump && has_back_jump);
}

#[test]
fn translates_for_loop_with_all_parts() {
    let out = compile_ir("for (var i = 0; i < 3; i++) { a += i; }");

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Const(ir::Const::Number(0.0))
            } if dest == "i"
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Classic {
                dest,
                function: ir::Function::Add(ir::Operand::Var(left), ir::Operand::Const(ir::Const::Number(1.0)))
            } if dest == "i" && left == "i"
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Classic {
                dest,
                function: ir::Function::Add(ir::Operand::Var(left), ir::Operand::Var(right))
            } if dest == "a" && left == "a" && right == "i"
        )
    }));
}

#[test]
fn translates_ternary_expression_assignment() {
    let out = compile_ir("var x = a ? 10 : 20;");

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::JumpIf(_, _)
            }
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Var(_)
            } if dest == "x"
        )
    }));
}

#[test]
fn translates_with_statement_scope_ops() {
    let out = compile_ir("with (obj) { a = b; }");

    let has_push = out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::PushToScope(_)
            }
        )
    });

    let has_pop = out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::PopFromScope()
            }
        )
    });

    assert!(has_push && has_pop);
}

#[test]
fn translates_function_declaration_and_call() {
    let out = compile_ir("function add(a, b) { return a + b; } add(1, 2);");

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::FnStart(name, 2)
            } if name == "add"
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::Return(Some(_))
            }
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::FnEnd()
            }
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::FnCall(name, _)
            } if name == "add"
        )
    }));
}

#[test]
fn translates_member_call_shape() {
    let out = compile_ir("obj.log('ok');");

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Call {
                function: ir::SoloFunction::FnCall(name, _)
            } if name == "obj_log"
        )
    }));
}

#[test]
fn translates_nested_mixed_constructs() {
    let out = compile_ir(
        "var sum = 0; for (var i = 0; i < 3; i++) { if (i == 1) { sum += i; } else { sum += 2; } }",
    );

    let label_count = out
        .iter()
        .filter(|instr| {
            matches!(
                instr,
                ir::Instruction::Call {
                    function: ir::SoloFunction::Label(_)
                }
            )
        })
        .count();

    let jumpif_count = out
        .iter()
        .filter(|instr| {
            matches!(
                instr,
                ir::Instruction::Call {
                    function: ir::SoloFunction::JumpIf(_, _)
                }
            )
        })
        .count();

    assert!(label_count >= 8);
    assert!(jumpif_count >= 2);
}

#[test]
fn translates_object_member_assignment_shape() {
    let out = compile_ir("obj.a = 1;");

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Const(ir::Const::Number(1.0))
            } if dest == "obj_a"
        )
    }));
}

#[test]
fn translates_dynamic_index_assignment_shape() {
    let out = compile_ir("obj[k] = v;");

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Var(src)
            } if dest == "obj_k" && src == "v"
        )
    }));
}

#[test]
fn translates_delete_member_to_undefined_and_true() {
    let out = compile_ir("var ok = delete obj.a;");

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Const(ir::Const::Undefined)
            } if dest == "obj_a"
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Const(ir::Const::Boolean(true))
            } if dest == "ok"
        )
    }));
}

#[test]
fn translates_prototype_style_paths() {
    let out = compile_ir("Foo.prototype.x = 1; var y = inst.x;");

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Const(ir::Const::Number(1.0))
            } if dest == "Foo_prototype_x"
        )
    }));

    assert!(out.iter().any(|instr| {
        matches!(
            instr,
            ir::Instruction::Assign {
                dest,
                src: ir::Operand::Var(src)
            } if dest == "y" && src == "inst_x"
        )
    }));
}

#[test]
fn translates_break_and_continue_with_loop_jumps() {
    let out = compile_ir("while (true) { continue; break; }");

    let jump_count = out
        .iter()
        .filter(|instr| {
            matches!(
                instr,
                ir::Instruction::Call {
                    function: ir::SoloFunction::Jump(_)
                }
            )
        })
        .count();

    assert!(jump_count >= 4);
}
