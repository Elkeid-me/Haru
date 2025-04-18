use super::Processor;
use super::tcg::Tcg::{self, *};
use llvm_ir::Constant;
use llvm_ir::instruction::{AShr, Add, And, LShr, Mul, Or, SDiv, SRem, Shl, Sub, UDiv, URem, Xor};
use llvm_ir::{Operand::*, Type::*, instruction::BinaryOp, instruction::HasResult};

fn sign_extend_const(original_bits: u64, n_bits: u32) -> i64 {
    (original_bits as i64) << (64 - n_bits) >> (64 - n_bits)
}

fn extract_const_i64(original: i64, n_bits: u32) -> i64 {
    original & ((0xffff_ffff_ffff_ffffu64 >> (64 - n_bits)) as i64)
}

fn extract_const_u64(original: u64, n_bits: u32) -> i64 {
    (original & (0xffff_ffff_ffff_ffffu64 >> (64 - n_bits))) as i64
}

/// 面善又友善的奇妙宏
///
/// 处理两个非 Const 操作数的情况
macro_rules! two_variables {
    ($processor: expr, $l_handler: expr, $r_handler: expr, $ret_handler: expr, $ret_bits: expr,
     $op_32: ident, $op_64: ident,
     $sign: literal) => {{
        let l_ty = $processor.symbol_table.borrow().get(&$l_handler).unwrap().clone();
        let r_ty = $processor.symbol_table.borrow().get(&$r_handler).unwrap().clone();
        match (l_ty.as_ref(), r_ty.as_ref()) {
            (IntegerType { bits: l_bits }, IntegerType { bits: r_bits }) if *l_bits == *r_bits && *l_bits == $ret_bits => {
                match l_bits {
                    0..32 => {
                        if $sign {
                            vec![
                                ShliI32 { ret: $l_handler, arg_1: $l_handler, arg_2: 32 - ($ret_bits as i32) },
                                SariI32 { ret: $l_handler, arg_1: $l_handler, arg_2: 32 - ($ret_bits as i32) },
                                ShliI32 { ret: $r_handler, arg_1: $r_handler, arg_2: 32 - ($ret_bits as i32) },
                                SariI32 { ret: $r_handler, arg_1: $r_handler, arg_2: 32 - ($ret_bits as i32) },
                                $op_32 { ret: $ret_handler, arg_1: $l_handler, arg_2: $r_handler },
                                ExtactI32 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                            ]
                        } else {
                            vec![
                                $op_32 { ret: $ret_handler, arg_1: $l_handler, arg_2: $r_handler },
                                ExtactI32 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                            ]
                        }
                    }
                    32 => vec![$op_32 { ret: $ret_handler, arg_1: $l_handler, arg_2: $r_handler }],
                    33..64 => {
                        if $sign {
                            vec![
                                ShliI64 { ret: $l_handler, arg_1: $l_handler, arg_2: 64 - ($ret_bits as i64) },
                                SariI64 { ret: $l_handler, arg_1: $l_handler, arg_2: 64 - ($ret_bits as i64) },
                                ShliI64 { ret: $r_handler, arg_1: $r_handler, arg_2: 64 - ($ret_bits as i64) },
                                SariI64 { ret: $r_handler, arg_1: $r_handler, arg_2: 64 - ($ret_bits as i64) },
                                $op_64 { ret: $ret_handler, arg_1: $l_handler, arg_2: $r_handler },
                                ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                            ]
                        } else {
                            vec![
                                $op_64 { ret: $ret_handler, arg_1: $l_handler, arg_2: $r_handler },
                                ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                            ]
                        }
                    }
                    64 => vec![$op_64 { ret: $ret_handler, arg_1: $l_handler, arg_2: $r_handler }],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

/// 处理一个非 Const 操作数和一个 Const 操作数的情况
macro_rules! vc_imm_tcg {
    ($processor: expr, $v_handler: expr, $constant: expr, $ret_handler: expr, $ret_bits: expr,
     $op_32_imm: ident, $op_64_imm: ident,
     $sign: literal) => {{
        let v_ty = $processor.symbol_table.borrow().get(&$v_handler).unwrap().clone();
        match (v_ty.as_ref(), $constant.as_ref()) {
            (IntegerType { bits: v_bits }, Constant::Int { bits: c_bits, value })
                if *v_bits == *c_bits && *v_bits == $ret_bits =>
            {
                match v_bits {
                    0..32 => {
                        if $sign {
                            vec![
                                ShliI32 { ret: $v_handler, arg_1: $v_handler, arg_2: 32 - ($ret_bits as i32) },
                                SariI32 { ret: $v_handler, arg_1: $v_handler, arg_2: 32 - ($ret_bits as i32) },
                                $op_32_imm {
                                    ret: $ret_handler,
                                    arg_1: $v_handler,
                                    arg_2: sign_extend_const(*value, $ret_bits) as i32,
                                },
                                ExtactI32 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                            ]
                        } else {
                            vec![
                                $op_32_imm { ret: $ret_handler, arg_1: $v_handler, arg_2: *value as i32 },
                                ExtactI32 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                            ]
                        }
                    }
                    32 => vec![$op_32_imm { ret: $ret_handler, arg_1: $v_handler, arg_2: *value as i32 }],
                    33..64 => {
                        if $sign {
                            vec![
                                ShliI64 { ret: $v_handler, arg_1: $v_handler, arg_2: 64 - ($ret_bits as i64) },
                                SariI64 { ret: $v_handler, arg_1: $v_handler, arg_2: 64 - ($ret_bits as i64) },
                                $op_64_imm {
                                    ret: $ret_handler,
                                    arg_1: $v_handler,
                                    arg_2: sign_extend_const(*value, $ret_bits),
                                },
                                ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                            ]
                        } else {
                            vec![
                                $op_64_imm { ret: $ret_handler, arg_1: $v_handler, arg_2: *value as i64 },
                                ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                            ]
                        }
                    }
                    64 => vec![$op_64_imm { ret: $ret_handler, arg_1: $v_handler, arg_2: *value as i64 }],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

macro_rules! const_variable {
    ($processor: expr, $v_handler: expr, $constant: expr, $ret_hdler: expr, $ret_bits: expr,
     $op_32: ident, $op_64: ident,
     $sign: literal, $const_first: literal) => {{
        let v_ty = $processor.symbol_table.borrow().get(&$v_handler).unwrap().clone();
        match (v_ty.as_ref(), $constant.as_ref()) {
            (IntegerType { bits: v_bits }, Constant::Int { bits: c_bits, value })
                if *v_bits == *c_bits && *v_bits == $ret_bits =>
            {
                match v_bits {
                    0..=32 => {
                        let mut ret = vec![MoviI32 { ret: $ret_hdler, arg: sign_extend_const(*value, $ret_bits) as i32 }];
                        if $const_first {
                            ret.extend(two_variables!(
                                $processor, $ret_hdler, $v_handler, $ret_hdler, $ret_bits, $op_32, $op_64, $sign
                            ));
                        } else {
                            ret.extend(two_variables!(
                                $processor, $v_handler, $ret_hdler, $ret_hdler, $ret_bits, $op_32, $op_64, $sign
                            ));
                        }
                        ret
                    }
                    33..=64 => {
                        let mut ret = vec![MoviI64 { ret: $ret_hdler, arg: sign_extend_const(*value, $ret_bits) }];
                        if $const_first {
                            ret.extend(two_variables!(
                                $processor, $ret_hdler, $v_handler, $ret_hdler, $ret_bits, $op_32, $op_64, $sign
                            ));
                        } else {
                            ret.extend(two_variables!(
                                $processor, $v_handler, $ret_hdler, $ret_hdler, $ret_bits, $op_32, $op_64, $sign
                            ));
                        }
                        ret
                    }
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

/// 处理两个 Const 操作数的情况
macro_rules! two_consts {
    ($op: tt, $l_constant: expr, $r_constant: expr, $ret_handler: expr, $ret_bits: expr, $sign: literal) => {{
        match ($l_constant.as_ref(), $r_constant.as_ref()) {
            (Constant::Int { bits: l_bits, value: l_value }, Constant::Int { bits: r_bits, value: r_value })
                if *l_bits == *r_bits && *l_bits == $ret_bits =>
            {
                match l_bits {
                    0..32 => {
                        if $sign {
                            vec![MoviI32 {
                                ret: $ret_handler,
                                arg: extract_const_i64(
                                    sign_extend_const(*l_value, $ret_bits) $op sign_extend_const(*r_value, $ret_bits),
                                    $ret_bits,
                                ) as i32,
                            }]
                        } else {
                            vec![MoviI32 { ret: $ret_handler, arg: extract_const_u64(l_value $op r_value, $ret_bits) as i32 }]
                        }
                    }
                    32 => vec![MoviI32 { ret: $ret_handler, arg: (l_value $op r_value) as i32 }],
                    33..64 => {
                        if $sign {
                            vec![MoviI64 {
                                ret: $ret_handler,
                                arg: extract_const_i64(
                                    sign_extend_const(*l_value, $ret_bits) $op sign_extend_const(*r_value, $ret_bits),
                                    $ret_bits,
                                ),
                            }]
                        } else {
                            vec![MoviI64 { ret: $ret_handler, arg: extract_const_u64(l_value $op r_value, $ret_bits) }]
                        }
                    }
                    64 => vec![MoviI64 { ret: $ret_handler, arg: (l_value $op r_value) as i64 }],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

macro_rules! int_op_impl {
    ($processor: expr, $inst: expr, $op: tt,
     $op_32: ident, $op_32_imm: ident, $op_32_imm_2: ident,
     $op_64: ident, $op_64_imm: ident, $op_64_imm_2: ident,
     $sign: literal, $imm_tcg: literal) => {{
        let ret_handler = $processor.name_to_handler($inst.get_result());
        let ret_bits = match $processor.symbol_table.borrow().get(&ret_handler).unwrap().as_ref() {
            IntegerType { bits } => *bits,
            _ => todo!(),
        };
        match ($inst.get_operand0(), $inst.get_operand1()) {
            (LocalOperand { name: l_name, ty: _ }, LocalOperand { name: r_name, ty: _ }) => {
                let l_handler = $processor.name_to_handler(l_name);
                let r_handler = $processor.name_to_handler(r_name);
                two_variables!($processor, l_handler, r_handler, ret_handler, ret_bits, $op_32, $op_64, $sign)
            }
            (LocalOperand { name: v_name, ty: _ }, ConstantOperand(constant)) => {
                let v_handler = $processor.name_to_handler(v_name);
                if $imm_tcg {
                    vc_imm_tcg!($processor, v_handler, constant, ret_handler, ret_bits, $op_32_imm, $op_64_imm, $sign)
                } else {
                    const_variable!($processor, v_handler, constant, ret_handler, ret_bits, $op_32, $op_64, $sign, false)
                }
            }
            (ConstantOperand(constant), LocalOperand { name: v_name, ty: _ }) => {
                let v_handler = $processor.name_to_handler(v_name);
                if $imm_tcg {
                    vc_imm_tcg!($processor, v_handler, constant, ret_handler, ret_bits, $op_32_imm_2, $op_64_imm_2, $sign)
                } else {
                    const_variable!($processor, v_handler, constant, ret_handler, ret_bits, $op_32, $op_64, $sign, true)
                }
            }
            (ConstantOperand(l_constant), ConstantOperand(r_constant)) => {
                two_consts!($op, l_constant, r_constant, ret_handler, ret_bits, $sign)
            }
            _ => todo!(),
        }
    }};
}

macro_rules! int_op {
    ($processor: expr, $inst: expr, $op: tt,
     $op_32: ident, $op_32_imm: ident,
     $op_64: ident, $op_64_imm: ident) => {
        int_op_impl!($processor, $inst, $op, $op_32, $op_32_imm, $op_32_imm, $op_64, $op_64_imm, $op_64_imm, false, true)
    };
    ($processor: expr, $inst: expr, $op: tt,
     $op_32: ident, $op_32_imm: ident, $op_32_imm_2: ident,
     $op_64: ident, $op_64_imm: ident, $op_64_imm_2: ident) => {
        int_op_impl!($processor, $inst, $op, $op_32, $op_32_imm, $op_32_imm_2, $op_64, $op_64_imm, $op_64_imm_2, false, true)
    };
    ($processor: expr, $inst: expr, $op: tt, $op_32: ident, $op_64: ident, $sign: literal) => {
        int_op_impl!($processor, $inst, $op, $op_32, AddiI32, AddiI32, $op_64, AddiI64, AddiI64, $sign, false)
    };
}

impl Processor<'_> {
    pub fn add(&self, add: &Add) -> Vec<Tcg> {
        int_op!(self, add, +, AddI32, AddiI32, AddI64, AddiI64)
    }

    pub fn sub(&self, sub: &Sub) -> Vec<Tcg> {
        int_op!(self, sub, -, SubI32, SubiI32, SubfiI32, SubI64, SubiI64, SubfiI64)
    }

    pub fn mul(&self, mul: &Mul) -> Vec<Tcg> {
        int_op!(self, mul, *, MulI32, MuliI32, MulI64, MuliI64)
    }

    pub fn sdiv(&self, sdiv: &SDiv) -> Vec<Tcg> {
        int_op!(self, sdiv, /, DivI32, DivI64, true)
    }

    pub fn udiv(&self, udiv: &UDiv) -> Vec<Tcg> {
        int_op!(self, udiv, /, DivuI32, DivuI64, false)
    }

    pub fn srem(&self, srem: &SRem) -> Vec<Tcg> {
        int_op!(self, srem, /, RemI32, RemI64, true)
    }

    pub fn urem(&self, urem: &URem) -> Vec<Tcg> {
        int_op!(self, urem, /, RemuI32, RemuI64, false)
    }

    pub fn and(&self, and: &And) -> Vec<Tcg> {
        int_op!(self, and, &, AndI32, AndiI32, AndI64, AndiI64)
    }

    pub fn or(&self, or: &Or) -> Vec<Tcg> {
        int_op!(self, or, |, OrI32, OriI32, OrI64, OriI64)
    }

    pub fn xor(&self, xor: &Xor) -> Vec<Tcg> {
        int_op!(self, xor, ^, XorI32, XoriI32, XorI64, XoriI64)
    }

    pub fn shl(&self, shl: &Shl) -> Vec<Tcg> {
        int_op!(self, shl, <<, ShlI32, ShlI64, false)
    }

    pub fn shr(&self, shl: &LShr) -> Vec<Tcg> {
        int_op!(self, shl, >>, ShrI32, ShrI64, false)
    }

    pub fn sar(&self, shl: &AShr) -> Vec<Tcg> {
        int_op!(self, shl, >>, SarI32, SarI64, true)
    }
}
