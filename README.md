# RIASM
RIASM stands for Rust Interpreter for ASM. It is a basic, asm-like language interpreter that allows you to write your own instructions and define your own registers, allowing you to embed this in for example some kind of game,
allowing users to interface with a basic interpreter. The big benefit of using RIASM is that it is extremely bare bones, it posses no native instructions of it's own, and has no default registers, the end user must define
every single one of these, allowing you to leverage it to define each instruction to mean whatever you like.

# How to build

	cargo build --release

# How To Use
Simply import 

	riasm::asm_definition::*;

and then use provided builder functions to construct and execute the asm interpreter you just built.
Example:
	
	use riasm::asm_definition::*;

	fn main() {
		let test_vec: Vec<ASTNode> = vec![
			ASTNode::ASTInstruction("MOV".into()),
			ASTNode::ASTRegister("R1".into()),
			ASTNode::ASTValue(1.into()),
			ASTNode::ASTExprEnd,
			ASTNode::ASTInstruction("MOV".into()),
			ASTNode::ASTRegister("R2".into()),
			ASTNode::ASTValue(1.into()),
			ASTNode::ASTExprEnd,
			ASTNode::ASTInstruction("ADD".into()),
			ASTNode::ASTRegister("R1".into()),
			ASTNode::ASTRegister("R2".into()),
			ASTNode::ASTExprEnd,
			ASTNode::ASTInstruction("PRINT".into()),
			ASTNode::ASTRegister("R1".into()),
			ASTNode::ASTExprEnd,
		];

		ASMDefinition::new()
			.insert_register("R1")
			.insert_register("R2")
			.insert_instruction("MOV", |arg| arg[0].try_modify_register(arg[1].resolve()))
			.insert_instruction("ADD", |arg| {
			arg[0].try_modify_register(arg[0].resolve() + arg[1].resolve())
			}).insert_instruction("PRINT", |arg|println!("{}",arg[0].resolve())).run(test_vec);
	}
	
	
This example will first push 1 to R1, push 1 to R2, sum them and place the sum in R1 and then display R1.
