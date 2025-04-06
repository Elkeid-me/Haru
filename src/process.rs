mod fp_op;
mod int_op;
mod tcg;

use tcg::Tcg;

use llvm_ir::{BasicBlock, Function, Instruction::*, Module, Name, Operand::*};
use llvm_ir::{Terminator::*, Type::*, TypeRef, types::Typed};
use std::{cell::RefCell, collections::HashMap};

type Handler = u64;

struct Counter {
    num: Handler,
}

impl Counter {
    fn new() -> Self {
        Self { num: 0 }
    }

    fn get(&mut self) -> Handler {
        self.num += 1;
        self.num
    }
}

pub struct Processor<'a> {
    counter: RefCell<Counter>,
    pub symbol_table: RefCell<HashMap<Handler, TypeRef>>,
    symbol_table_2: RefCell<HashMap<Name, Handler>>,
    ret: Handler,
    pub parameters: Vec<Handler>,
    module: &'a Module,
    basic_blocks: RefCell<HashMap<Name, Handler>>,
}

impl<'a> Processor<'a> {
    pub fn new(module: &'a Module) -> Self {
        Self {
            counter: RefCell::new(Counter::new()),
            ret: 0,
            symbol_table: RefCell::new(HashMap::new()),
            symbol_table_2: RefCell::new(HashMap::new()),
            parameters: Vec::new(),
            module,
            basic_blocks: RefCell::new(HashMap::new()),
        }
    }

    fn process_basic_block(&self, block: &BasicBlock) -> Vec<Tcg> {
        let mut result = vec![/*Tcg::Label(*self.basic_blocks.borrow().get(&block.name).unwrap())*/];
        let mut insts = block
            .instrs
            .iter()
            .map(|inst| match inst {
                Add(add) => self.add(add),
                Sub(sub) => self.sub(sub),
                Mul(mul) => self.mul(mul),
                UDiv(udiv) => self.udiv(udiv),
                SDiv(sdiv) => self.sdiv(sdiv),
                URem(urem) => self.urem(urem),
                SRem(srem) => self.srem(srem),
                And(and) => self.and(and),
                Or(or) => self.or(or),
                Xor(xor) => self.xor(xor),
                Shl(_shl) => todo!(),
                LShr(lshr) => todo!(),
                AShr(ashr) => todo!(),
                FAdd(fadd) => self.fadd(fadd),
                FSub(fsub) => self.fsub(fsub),
                FMul(fmul) => self.fmul(fmul),
                FDiv(fdiv) => self.fdiv(fdiv),
                FRem(frem) => todo!(),
                FNeg(fneg) => self.fneg(fneg),
                _ => vec![],
            })
            .flatten()
            .collect::<Vec<_>>();

        result.append(&mut insts);
        let mut ret = match &block.term {
            Ret(r) => match &r.return_operand {
                Some(LocalOperand { name, ty }) => match ty.as_ref() {
                    IntegerType { bits: 0..=32 } => vec![
                        Tcg::ExtI32I64 { ret: self.ret, arg: *self.symbol_table_2.borrow().get(name).unwrap() },
                        Tcg::SetDestGpr { expr: self.ret },
                        Tcg::Ret,
                    ],
                    IntegerType { bits: 33..=64 } => vec![
                        Tcg::MovI64 { ret: self.ret, arg: *self.symbol_table_2.borrow().get(name).unwrap() },
                        Tcg::SetDestGpr { expr: self.ret },
                        Tcg::Ret,
                    ],
                    FPType(llvm_ir::types::FPType::Double) => vec![
                        Tcg::MovI64 { ret: self.ret, arg: *self.symbol_table_2.borrow().get(name).unwrap() },
                        Tcg::SetDestFprD { expr: self.ret },
                        Tcg::Ret,
                    ],
                    FPType(llvm_ir::types::FPType::Single) => vec![
                        Tcg::MovI64 { ret: self.ret, arg: *self.symbol_table_2.borrow().get(name).unwrap() },
                        Tcg::SetDestFprHs { expr: self.ret },
                        Tcg::Ret,
                    ],
                    _ => todo!(),
                },
                Some(ConstantOperand(constant)) => match constant.as_ref() {
                    llvm_ir::Constant::Int { bits, value } => vec![
                        Tcg::MoviI64 { ret: self.ret, arg: *value as i64 },
                        Tcg::ExtactI64 { ret: self.ret, arg: self.ret, pos: 0, len: *bits },
                        Tcg::SetDestGpr { expr: self.ret },
                        Tcg::Ret,
                    ],
                    llvm_ir::Constant::Float(llvm_ir::constant::Float::Single(single)) => vec![
                        Tcg::MoviI64 { ret: self.ret, arg: single.to_bits() as i64 },
                        Tcg::SetDestFprHs { expr: self.ret },
                        Tcg::Ret,
                    ],
                    llvm_ir::Constant::Float(llvm_ir::constant::Float::Double(double)) => vec![
                        Tcg::MoviI64 { ret: self.ret, arg: double.to_bits() as i64 },
                        Tcg::SetDestFprD { expr: self.ret },
                        Tcg::Ret,
                    ],
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        };
        result.append(&mut ret);
        result
    }

    pub fn process_func(&mut self, func: &Function) -> Vec<Tcg> {
        let ret_handler = self.counter.borrow_mut().get();
        let ret_type = func.return_type.clone();
        self.ret = ret_handler;
        self.symbol_table.borrow_mut().insert(ret_handler, ret_type);

        for parameter in func.parameters.iter() {
            let para_handler = self.counter.borrow_mut().get();
            let para_type = parameter.ty.clone();
            self.symbol_table.borrow_mut().insert(para_handler, para_type);
            self.symbol_table_2.borrow_mut().insert(parameter.name.clone(), para_handler);
            self.parameters.push(para_handler);
        }

        for block in func.basic_blocks.iter() {
            self.basic_blocks.borrow_mut().insert(block.name.clone(), self.counter.borrow_mut().get());
            for inst in block.instrs.iter() {
                if let Some(name) = inst.try_get_result() {
                    let ty = inst.get_type(&self.module.types);
                    let handler = self.counter.borrow_mut().get();
                    self.symbol_table.borrow_mut().insert(handler, ty);
                    self.symbol_table_2.borrow_mut().insert(name.clone(), handler);
                }
            }
        }

        func.basic_blocks.iter().map(|block| self.process_basic_block(block)).flatten().collect()
    }
}
