mod cvt_op;
mod fp_op;
mod int_op;
mod tcg;

use llvm_ir::instruction::{HasResult, Select};
use llvm_ir::{BasicBlock, Function, Instruction::*, Module, Name, Operand::*, Terminator::*};
use llvm_ir::{Type::*, TypeRef, function::ParameterAttribute, types::Typed, types::Types};
use std::collections::HashSet;
use std::{cell::RefCell, collections::HashMap};
use tcg::Tcg;

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
    pub used: RefCell<HashSet<Handler>>,
    symbol_table_2: RefCell<HashMap<Name, Handler>>,
    pub ret: Handler,
    pub parameters: Vec<Handler>,
    module: &'a Module,
    basic_blocks: RefCell<HashMap<Name, Handler>>,
    ret_attr: Option<ParameterAttribute>,
    pub use_float: bool,
    tmps: RefCell<[Option<Handler>; 8]>,
}

macro_rules! const_value {
    ($constant: expr) => {
        match $constant {
            llvm_ir::Constant::Int { bits: _, value } => *value as i64,
            llvm_ir::Constant::Float(llvm_ir::constant::Float::Single(single)) => {
                (single.to_bits() as u64 | 0xffff_ffff_0000_0000) as i64
            }
            llvm_ir::Constant::Float(llvm_ir::constant::Float::Double(double)) => double.to_bits() as i64,
            _ => todo!(),
        }
    };
}

macro_rules! move_value {
    ($prc: expr, $value: expr, $ret_handler: expr) => {
        match $value {
            LocalOperand { name, ty: _ } => {
                let v_handler = $prc.name_to_handler(name);
                $prc.use_variable(v_handler);
                vec![Tcg::MovI64 { ret: $ret_handler, arg: v_handler }]
            }
            ConstantOperand(constant) => vec![Tcg::MoviI64 { ret: $ret_handler, arg: const_value!(constant.as_ref()) }],
            _ => todo!(),
        }
    };
}

impl<'a> Processor<'a> {
    fn select(&self, select: &Select) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(select.get_result());
        self.use_variable(ret_handler);
        match &select.condition {
            LocalOperand { name, ty } if matches!(ty.as_ref(), IntegerType { bits: 1 }) => {
                let cond_handler = self.name_to_handler(name);
                let tmp_1 = self.get_tmp::<1>();
                self.use_variable(cond_handler);
                self.use_variable(tmp_1);

                let mut ret = vec![Tcg::MoviI64 { ret: tmp_1, arg: 1 }];

                match (&select.true_value, &select.false_value) {
                    (LocalOperand { name: l_name, ty: _ }, LocalOperand { name: r_name, ty: _ }) => {
                        let l_handler = self.name_to_handler(l_name);
                        let r_handler = self.name_to_handler(r_name);
                        self.use_variable(l_handler);
                        self.use_variable(r_handler);
                        ret.extend(vec![Tcg::MovcondI64 {
                            ret: ret_handler,
                            cond_1: cond_handler,
                            cond_2: tmp_1,
                            arg_1: l_handler,
                            arg_2: r_handler,
                        }]);
                        ret
                    }
                    (LocalOperand { name, ty: _ }, ConstantOperand(constant)) => {
                        let l_handler = self.name_to_handler(name);
                        let r_handler = self.get_tmp::<2>();
                        self.use_variable(l_handler);
                        self.use_variable(r_handler);
                        ret.extend(vec![
                            Tcg::MoviI64 { ret: r_handler, arg: const_value!(constant.as_ref()) },
                            Tcg::MovcondI64 {
                                ret: ret_handler,
                                cond_1: cond_handler,
                                cond_2: tmp_1,
                                arg_1: l_handler,
                                arg_2: r_handler,
                            },
                        ]);
                        ret
                    }
                    (ConstantOperand(constant), LocalOperand { name, ty: _ }) => {
                        let l_handler = self.get_tmp::<2>();
                        let r_handler = self.name_to_handler(name);
                        self.use_variable(l_handler);
                        self.use_variable(r_handler);
                        ret.extend(vec![
                            Tcg::MoviI64 { ret: l_handler, arg: const_value!(constant.as_ref()) },
                            Tcg::MovcondI64 {
                                ret: ret_handler,
                                cond_1: cond_handler,
                                cond_2: tmp_1,
                                arg_1: l_handler,
                                arg_2: r_handler,
                            },
                        ]);
                        ret
                    }
                    (ConstantOperand(l_constant), ConstantOperand(r_constant)) => {
                        let l_handler = self.get_tmp::<2>();
                        let r_handler = self.get_tmp::<3>();
                        self.use_variable(l_handler);
                        self.use_variable(r_handler);
                        ret.extend(vec![
                            Tcg::MoviI64 { ret: l_handler, arg: const_value!(l_constant.as_ref()) },
                            Tcg::MoviI64 { ret: r_handler, arg: const_value!(r_constant.as_ref()) },
                            Tcg::MovcondI64 {
                                ret: ret_handler,
                                cond_1: cond_handler,
                                cond_2: tmp_1,
                                arg_1: l_handler,
                                arg_2: r_handler,
                            },
                        ]);
                        ret
                    }
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                llvm_ir::Constant::Int { bits: 1, value: 1 } => move_value!(self, &select.true_value, ret_handler),
                llvm_ir::Constant::Int { bits: 1, value: 0 } => move_value!(self, &select.false_value, ret_handler),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    #[inline]
    pub fn get_tmp<const N: usize>(&self) -> Handler {
        if matches!(self.tmps.borrow()[N - 1].as_ref(), None) {
            let new_handler = self.new_handler();
            self.tmps.borrow_mut()[N - 1] = Some(new_handler);
            self.symbol_table.borrow_mut().insert(new_handler, Types::i64(&self.module.types));
        }
        self.tmps.borrow()[N - 1].unwrap()
    }

    #[inline]
    pub fn new_handler(&self) -> Handler {
        self.counter.borrow_mut().get()
    }

    #[inline]
    pub fn name_to_handler(&self, name: &Name) -> Handler {
        *self.symbol_table_2.borrow().get(name).unwrap()
    }

    #[inline]
    pub fn use_variable(&self, handler: Handler) {
        self.used.borrow_mut().insert(handler);
    }

    pub fn new(module: &'a Module) -> Self {
        Self {
            counter: RefCell::new(Counter::new()),
            ret: 0,
            symbol_table: RefCell::default(),
            symbol_table_2: RefCell::default(),
            used: RefCell::default(),
            parameters: Vec::new(),
            module,
            basic_blocks: RefCell::default(),
            ret_attr: None,
            use_float: false,
            tmps: RefCell::default(),
        }
    }

    fn process_basic_block(&self, block: &BasicBlock) -> Vec<Tcg> {
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
                Shl(shl) => self.shl(shl),
                LShr(lshr) => self.shr(lshr),
                AShr(ashr) => self.sar(ashr),
                FAdd(fadd) => self.fadd(fadd),
                FSub(fsub) => self.fsub(fsub),
                FMul(fmul) => self.fmul(fmul),
                FDiv(fdiv) => self.fdiv(fdiv),
                FRem(frem) => self.frem(frem),
                FNeg(fneg) => self.fneg(fneg),
                FPToSI(fp_to_si) => self.fp_to_si(fp_to_si),
                FPToUI(fp_to_ui) => self.fp_to_ui(fp_to_ui),
                SIToFP(si_to_fp) => self.si_to_fp(si_to_fp),
                UIToFP(ui_to_fp) => self.ui_to_fp(ui_to_fp),
                FPExt(fp_ext) => self.fp_ext(fp_ext),
                FPTrunc(fp_trunc) => self.fp_trunc(fp_trunc),
                ZExt(zext) => self.zext(zext),
                SExt(sext) => self.sext(sext),
                Trunc(trunc) => self.trunc(trunc),
                ICmp(icmp) => self.icmp(icmp),
                Select(select) => self.select(select),
                _ => todo!(),
            })
            .flatten()
            .collect::<Vec<_>>();

        let mut ret = match &block.term {
            Ret(r) => match &r.return_operand {
                Some(LocalOperand { name, ty }) => {
                    let arg_handler = self.name_to_handler(name);
                    self.use_variable(arg_handler);
                    match ty.as_ref() {
                        IntegerType { bits } if matches!(bits, 0..=32) => {
                            let mut ret = vec![Tcg::rv_arc(
                                Tcg::ExtrlI64I32 { ret: self.ret, arg: arg_handler },
                                Tcg::MovI64 { ret: self.ret, arg: arg_handler },
                            )];
                            if matches!(self.ret_attr, Some(ParameterAttribute::SignExt)) {
                                ret.extend([
                                    Tcg::rv_arc(
                                        Tcg::ShliI32 { ret: self.ret, arg_1: self.ret, arg_2: (32 - bits) as i32 },
                                        Tcg::ShliI64 { ret: self.ret, arg_1: self.ret, arg_2: (64 - bits) as i64 },
                                    ),
                                    Tcg::rv_arc(
                                        Tcg::SariI32 { ret: self.ret, arg_1: self.ret, arg_2: (32 - bits) as i32 },
                                        Tcg::SariI64 { ret: self.ret, arg_1: self.ret, arg_2: (64 - bits) as i64 },
                                    ),
                                ]);
                            }
                            ret.extend([Tcg::SetDestGpr { expr: self.ret }, Tcg::Ret { float: self.use_float }]);
                            ret
                        }
                        IntegerType { bits } if matches!(bits, 33..=64) => {
                            let mut ret = vec![Tcg::MovI64 { ret: self.ret, arg: arg_handler }];
                            if matches!(self.ret_attr, Some(ParameterAttribute::SignExt)) {
                                ret.extend([
                                    Tcg::ShliI64 { ret: self.ret, arg_1: self.ret, arg_2: (64 - bits) as i64 },
                                    Tcg::SariI64 { ret: self.ret, arg_1: self.ret, arg_2: (64 - bits) as i64 },
                                ]);
                            }
                            ret.extend([
                                Tcg::rv_arc(Tcg::SetDestGprPair { expr: self.ret }, Tcg::SetDestGpr { expr: self.ret }),
                                Tcg::Ret { float: self.use_float },
                            ]);
                            ret
                        }
                        FPType(llvm_ir::types::FPType::Double) => vec![
                            Tcg::MovI64 { ret: self.ret, arg: arg_handler },
                            Tcg::SetDestFprD { expr: self.ret },
                            Tcg::Ret { float: self.use_float },
                        ],
                        FPType(llvm_ir::types::FPType::Single) => vec![
                            Tcg::MovI64 { ret: self.ret, arg: arg_handler },
                            Tcg::OriI64 { ret: self.ret, arg_1: self.ret, arg_2: 0xffff_ffff_0000_0000u64 as i64 },
                            Tcg::SetDestFprHs { expr: self.ret },
                            Tcg::Ret { float: self.use_float },
                        ],
                        _ => todo!(),
                    }
                }
                Some(ConstantOperand(constant)) => match constant.as_ref() {
                    llvm_ir::Constant::Int { bits, value } => match bits {
                        0..=32 => vec![
                            Tcg::rv_arc(
                                Tcg::MoviI32 { ret: self.ret, arg: *value as i32 },
                                Tcg::MoviI64 { ret: self.ret, arg: *value as i64 },
                            ),
                            Tcg::SetDestGpr { expr: self.ret },
                            Tcg::Ret { float: self.use_float },
                        ],
                        33..=64 => vec![
                            Tcg::MoviI64 { ret: self.ret, arg: *value as i64 },
                            Tcg::rv_arc(Tcg::SetDestGprPair { expr: self.ret }, Tcg::SetDestGpr { expr: self.ret }),
                            Tcg::Ret { float: self.use_float },
                        ],
                        _ => todo!(),
                    },
                    llvm_ir::Constant::Float(llvm_ir::constant::Float::Single(single)) => vec![
                        Tcg::MoviI64 { ret: self.ret, arg: (single.to_bits() as u64 | 0xffff_ffff_0000_0000) as i64 },
                        Tcg::SetDestFprHs { expr: self.ret },
                        Tcg::Ret { float: self.use_float },
                    ],
                    llvm_ir::Constant::Float(llvm_ir::constant::Float::Double(double)) => vec![
                        Tcg::MoviI64 { ret: self.ret, arg: double.to_bits() as i64 },
                        Tcg::SetDestFprD { expr: self.ret },
                        Tcg::Ret { float: self.use_float },
                    ],
                    llvm_ir::Constant::Poison(_) => vec![
                        Tcg::rv_arc(Tcg::MoviI32 { ret: self.ret, arg: 0 }, Tcg::MoviI64 { ret: self.ret, arg: 0 }),
                        Tcg::SetDestGpr { expr: self.ret },
                        Tcg::Ret { float: self.use_float },
                    ],
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        };
        insts.append(&mut ret);
        insts
    }

    pub fn process_func(&mut self, func: &Function) -> Vec<Tcg> {
        let ret_handler = self.new_handler();
        self.ret = ret_handler;
        self.symbol_table.borrow_mut().insert(ret_handler, func.return_type.clone());

        if matches!(func.return_type.as_ref(), llvm_ir::types::Type::FPType(_)) {
            self.use_float = true;
        }

        for parameter in func.parameters.iter() {
            let para_handler = self.new_handler();
            let para_type = parameter.ty.clone();
            if matches!(para_type.as_ref(), llvm_ir::types::Type::FPType(_)) {
                self.use_float = true;
            }
            self.symbol_table.borrow_mut().insert(para_handler, para_type);
            self.symbol_table_2.borrow_mut().insert(parameter.name.clone(), para_handler);
            self.parameters.push(para_handler);
        }

        for attr in func.return_attributes.iter() {
            if matches!(attr, ParameterAttribute::ZeroExt | ParameterAttribute::SignExt) {
                self.ret_attr = Some(attr.clone())
            }
        }

        for block in func.basic_blocks.iter() {
            self.basic_blocks.borrow_mut().insert(block.name.clone(), self.new_handler());
            for inst in block.instrs.iter() {
                if let Some(name) = inst.try_get_result() {
                    let ty = inst.get_type(&self.module.types);
                    if matches!(ty.as_ref(), llvm_ir::types::Type::FPType(_)) {
                        self.use_float = true;
                    }
                    let handler = self.new_handler();
                    self.symbol_table.borrow_mut().insert(handler, ty);
                    self.symbol_table_2.borrow_mut().insert(name.clone(), handler);
                }
            }
        }

        func.basic_blocks.iter().map(|block| self.process_basic_block(block)).flatten().collect()
    }
}
