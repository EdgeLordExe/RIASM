use std::collections::HashMap;

use crate::{
    asm_instruction::ASMInstruction,
    asm_value::{ASMValue, ASMValueHolder},
};

#[derive(Clone)]
pub enum ASTNode {
    ASTValue(ASMValue),
    ASTInstruction(String),
    ASTRegister(String),
    ASTExprEnd,
}

pub struct ASMDefinition {
    pub registers: HashMap<String, ASMValue>,
    pub instructions: HashMap<String, ASMInstruction>,
    pub labels: HashMap<String, usize>,
    _priority: u16,
    ptr_to_self: Option<*mut ASMDefinition>,
    errors: u64,
    halted: bool,
    current_line: usize,
}

impl ASMDefinition {
    pub fn dump_state(&self) {
        println!("== ASMDefinition STATE DUMP BEGIN ==");
        for (regs_name, reg_val) in self.registers.iter() {
            println!("REGISTER {0} is {1}", regs_name, reg_val);
        }
        for (instruction_name, _instruction) in self.instructions.iter() {
            println!("FOUND INSTRUCTION: {}", instruction_name);
        }
        println!("== ASMDefinition STATE DUMP END ==")
    }

    pub fn new() -> Self {
        let mut def = ASMDefinition {
            registers: HashMap::new(),
            instructions: HashMap::new(),
            labels: HashMap::new(),
            _priority: 1,
            ptr_to_self: None,
            errors: 0,
            halted: false,
            current_line: 0,
        };
        let def_ptr: *mut ASMDefinition = &mut def;
        def.ptr_to_self = Some(def_ptr);
        def
    }

    pub fn insert_register(mut self, reg_name: &str) -> Self {
        self.registers.insert(
            reg_name.into(),
            ASMValue::new_empty(self.ptr_to_self.clone()),
        );
        self
    }

    pub fn insert_instruction(
        mut self,
        instruction_name: &str,
        closure: fn(&mut ASMDefinition, Vec<ASMValue>),
    ) -> Self {
        self.instructions.insert(
            instruction_name.into(),
            ASMInstruction::new(closure, self.ptr_to_self.unwrap().clone()),
        );
        self
    }

    pub fn raise_exception(&mut self, error_message: &str, halt_execution: bool) {
        println!("{}", error_message);
        if halt_execution {
            self.halted = true;
        }
        self.errors += 1;
    }

    pub fn run(&mut self, token_stream: Vec<ASTNode>) {
        let mut current_instruction: Option<ASMInstruction> = None;
        let mut current_args: Vec<ASMValue> = Vec::new();
        while self.current_line < token_stream.len() {
            let token: ASTNode = token_stream[self.current_line].clone();
            if self.halted {
                return;
            }
            match token {
                ASTNode::ASTValue(value) => {
                    if current_instruction.is_none() {
                        self.raise_exception(
                            "ASTValue encountered with no instruction present",
                            true,
                        );
                        continue;
                    }
                    current_args.push(value.clone());
                }
                ASTNode::ASTInstruction(instruction) => {
                    if current_instruction.is_some() {
                        self.raise_exception(
                            "ASTInstruction encountered when another instruction is called",
                            true,
                        );
                        continue;
                    }
                    let instruction_ref = match self.instructions.get(&instruction) {
                        Some(reference) => reference,
                        None => {
                            self.raise_exception("Not a valid instruction", true);
                            continue;
                        }
                    };
                    current_instruction = Some((*instruction_ref).clone());
                }
                ASTNode::ASTRegister(reference) => {
                    if current_instruction.is_none() {
                        self.raise_exception(
                            "Register reference encountered with no instruction present",
                            false,
                        );
                        continue;
                    }
                    let register_ref = match self.registers.get(&reference) {
                        Some(_) => ASMValue::new_reg(reference.clone(), self.ptr_to_self.clone()),
                        None => {
                            self.raise_exception("Register not defined in ASMDefinition", true);
                            continue;
                        }
                    };

                    current_args.push(register_ref);
                }
                ASTNode::ASTExprEnd => match current_instruction {
                    Some(instruction) => {
                        instruction.call(self, current_args.clone());
                        current_instruction = None;
                        current_args.clear();
                    }
                    None => {}
                },
            }
            self.current_line += 1;
        }
    }

    pub fn jump_to_value(&mut self, value: ASMValue) {
        if let ASMValueHolder::Int(inner_value) = value.get_value_holder() {
            self.jump(inner_value as usize);
            return;
        }
        self.raise_exception("Invalid value of provided destination!", true);
    }

    pub fn jump_to_label(&mut self, label: ASMValue) {
        if let ASMValueHolder::Label(label_string) = label.get_value_holder() {
            if self.labels.contains_key(&label_string) {
                self.jump(*self.labels.get(&label_string).unwrap());
                return;
            }
        }
        self.raise_exception("Invalid label provided!", true);
    }

    pub fn jump(&mut self, destination: usize) {
        self.current_line = destination - 1;
    }

    pub fn scan(&mut self, code: String) -> Vec<ASTNode> {
        let mut output: Vec<ASTNode> = Vec::new();
        let lines: Vec<String> = code.split("\n").map(|x| x.to_string()).collect();

        for line in lines.iter() {
            let mut usable_line: String = line.clone();
            if usable_line.find(";;").is_some() {
                usable_line = usable_line.split_once(";;").unwrap().0.into();
            }
            usable_line = usable_line.trim_end().into();
            if usable_line.len() == 0 {
                continue;
            }
            let mut words: Vec<String> = usable_line.split(" ").map(|x| x.to_string()).collect();
            if words[0].chars().last().unwrap() == ':' {
                let mut label = words[0].clone();
                label.retain(|c| c != ':');
                self.labels.insert(label, output.len());
                continue;
            }
            output.push(self.match_instruction(words[0].clone()));
            words.remove(0);
            words
                .iter()
                .for_each(|word| output.push(self.match_argument(word.clone())));
            output.push(ASTNode::ASTExprEnd);
        }
        output
    }

    fn match_instruction(&mut self, mut word: String) -> ASTNode {
        word.retain(|c| !c.is_whitespace());
        if !self.instructions.contains_key(&word) {
            self.raise_exception(format!("{} is an unknown instruction", word).as_str(), true);
        }
        ASTNode::ASTInstruction(word)
    }

    fn match_argument(&mut self, mut word: String) -> ASTNode {
        word.retain(|c| !c.is_whitespace());
        if word.len() == 0 {
            self.raise_exception("Empty argument!", true);
            return ASTNode::ASTExprEnd;
        }
        if word.chars().all(|c| c.is_numeric()) {
            return ASTNode::ASTValue(word.parse::<i32>().unwrap().into());
        }
        if word.chars().all(|c| c.is_alphanumeric()) {
            return ASTNode::ASTValue(ASMValue::new_label(word, None));
        }
        if word.chars().next().unwrap() == '[' && word.chars().last().unwrap() == ']' {
            word.retain(|c| c.is_alphanumeric());
            println!("{}", word);
            return ASTNode::ASTRegister(word);
        }
        if word.chars().last().unwrap() == '"' && word.chars().next().unwrap() == '"' {
            todo!()
        }

        ASTNode::ASTExprEnd
    }

    pub fn interpret(&mut self, code: String) {
        let ast = self.scan(code);
        self.run(ast);
    }
}
