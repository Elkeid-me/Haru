use super::Processor;
use super::tcg::Tcg::{self, *};
use llvm_ir::Constant;
use llvm_ir::instruction::{Add, And, Mul, Or, Xor};
use llvm_ir::types::Types;
use llvm_ir::{Operand::*, Type::*, instruction::BinaryOp, instruction::HasResult};

/// 可交换的整数运算符
macro_rules! com_int_op_check {
    ($processor: expr, $inst: expr, $op_32: ident, $op_32_imm: ident, $op_64: ident, $op_64_imm: ident) => {{
        let ret_handler = *$processor.symbol_table_2.borrow().get($inst.get_result()).unwrap();
        let ret_bits = match *$processor.symbol_table.borrow().get(&ret_handler).unwrap().as_ref() {
            IntegerType { bits } => bits,
            _ => unreachable!(),
        };
        match ($inst.get_operand0(), $inst.get_operand1()) {
            (LocalOperand { name: l_name, ty: l_ty }, LocalOperand { name: r_name, ty: r_ty }) => {
                let l_handler = *$processor.symbol_table_2.borrow().get(l_name).unwrap();
                let r_handler = *$processor.symbol_table_2.borrow().get(r_name).unwrap();
                match (l_ty.as_ref(), r_ty.as_ref()) {
                    (IntegerType { bits: l_bits }, IntegerType { bits: r_bits }) if l_bits == r_bits => {
                        match (l_bits, ret_bits) {
                            (0..=32, 0..=32) => {
                                vec![
                                    $op_32 { ret: ret_handler, arg_1: l_handler, arg_2: r_handler },
                                    ExtactI32 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                                ]
                            }
                            (33..=64, 33..=64) => {
                                vec![
                                    $op_64 { ret: ret_handler, arg_1: l_handler, arg_2: r_handler },
                                    ExtactI64 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                                ]
                            }
                            (0..=32, 33..=64) => {
                                let tmp_handler = $processor.counter.borrow_mut().get();
                                $processor.symbol_table.borrow_mut().insert(tmp_handler, Types::i32(&$processor.module.types));
                                vec![
                                    $op_32 { ret: tmp_handler, arg_1: l_handler, arg_2: r_handler },
                                    ExtI32I64 { ret: ret_handler, arg: tmp_handler },
                                    ExtactI64 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                                ]
                            }
                            (33..=64, 0..=32) => {
                                let tmp_handler = $processor.counter.borrow_mut().get();
                                $processor.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&$processor.module.types));
                                vec![
                                    $op_64 { ret: tmp_handler, arg_1: l_handler, arg_2: r_handler },
                                    ExtrlI64I32 { ret: ret_handler, arg: tmp_handler },
                                    ExtactI32 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                                ]
                            }
                            _ => todo!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (LocalOperand { name: v_name, ty: v_ty }, ConstantOperand(constant))
            | (ConstantOperand(constant), LocalOperand { name: v_name, ty: v_ty }) => {
                let v_handler = *$processor.symbol_table_2.borrow().get(v_name).unwrap();
                match (v_ty.as_ref(), constant.as_ref()) {
                    (IntegerType { bits: v_bits }, Constant::Int { bits: c_bits, value }) if v_bits == c_bits => {
                        match (v_bits, ret_bits) {
                            (0..=32, 0..=32) => {
                                vec![
                                    $op_32_imm { ret: ret_handler, arg_1: v_handler, arg_2: *value as i32 },
                                    ExtactI32 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                                ]
                            }
                            (33..=64, 33..=64) => {
                                vec![
                                    $op_64_imm { ret: ret_handler, arg_1: v_handler, arg_2: *value as i64 },
                                    ExtactI64 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                                ]
                            }
                            (0..=32, 33..=64) => {
                                let tmp_handler = $processor.counter.borrow_mut().get();
                                $processor.symbol_table.borrow_mut().insert(tmp_handler, Types::i32(&$processor.module.types));
                                vec![
                                    $op_32_imm { ret: tmp_handler, arg_1: v_handler, arg_2: *value as i32 },
                                    ExtI32I64 { ret: ret_handler, arg: tmp_handler },
                                    ExtactI64 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                                ]
                            }
                            (33..=64, 0..=32) => {
                                let tmp_handler = $processor.counter.borrow_mut().get();
                                $processor.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&$processor.module.types));
                                vec![
                                    $op_64_imm { ret: tmp_handler, arg_1: v_handler, arg_2: *value as i64 },
                                    ExtrlI64I32 { ret: ret_handler, arg: tmp_handler },
                                    ExtactI32 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                                ]
                            }
                            _ => todo!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (ConstantOperand(l_constant), ConstantOperand(r_constant)) => match (l_constant.as_ref(), r_constant.as_ref()) {
                (Constant::Int { bits: l_bits, value: l_value }, Constant::Int { bits: r_bits, value: r_value })
                    if *l_bits == *r_bits =>
                {
                    match ret_bits {
                        0..=32 => vec![
                            MoviI32 { ret: ret_handler, arg: (l_value + r_value) as i32 },
                            ExtactI32 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                        ],

                        33..=64 => vec![
                            MoviI64 { ret: ret_handler, arg: (l_value + r_value) as i64 },
                            ExtactI64 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                        ],

                        _ => todo!(),
                    }
                }
                _ => unreachable!(),
            },
            _ => todo!(),
        }
    }};
}

impl Processor<'_> {
    pub fn add(&self, add: &Add) -> Vec<Tcg> {
        com_int_op_check!(self, add, AddI32, AddiI32, AddI64, AddiI64)
    }

    pub fn and(&self, and: &And) -> Vec<Tcg> {
        com_int_op_check!(self, and, AndI32, AndiI32, AndI64, AndiI64)
    }

    pub fn mul(&self, or: &Mul) -> Vec<Tcg> {
        com_int_op_check!(self, or, MulI32, MuliI32, MulI64, MuliI64)
    }

    pub fn or(&self, or: &Or) -> Vec<Tcg> {
        com_int_op_check!(self, or, OrI32, OriI32, OrI64, OriI64)
    }

    pub fn xor(&self, xor: &Xor) -> Vec<Tcg> {
        com_int_op_check!(self, xor, XorI32, XoriI32, XorI64, XoriI64)
    }
}
