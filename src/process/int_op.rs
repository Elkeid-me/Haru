use super::Processor;
use super::tcg::Tcg::{self, *};
use llvm_ir::Constant;
use llvm_ir::instruction::{AShr, Add, And, LShr, Mul, Or, SDiv, SRem, Shl, Sub, UDiv, URem, Xor};
use llvm_ir::{Operand::*, Type::*, instruction::BinaryOp, instruction::HasResult};

#[inline]
pub fn sign_extend_const(original_bits: u64, n_bits: u32) -> i64 {
    (original_bits as i64) << (64 - n_bits) >> (64 - n_bits)
}

#[inline]
pub fn extract_const_i64(original: i64, n_bits: u32) -> i64 {
    original & ((u64::MAX >> (64 - n_bits)) as i64)
}

#[inline]
pub fn extract_const_u64(original: u64, n_bits: u32) -> i64 {
    (original & (u64::MAX >> (64 - n_bits))) as i64
}

/// 面善又友善的奇妙宏
///
/// 处理两个非 Const 操作数的情况
macro_rules! two_variables {
    ($processor: expr, $l_handler: expr, $r_handler: expr, $ret_handler: expr, $ret_bits: expr,
     $op_64: ident,
     $sign: literal) => {{
        let l_ty = $processor.symbol_table.borrow().get(&$l_handler).unwrap().clone();
        let r_ty = $processor.symbol_table.borrow().get(&$r_handler).unwrap().clone();
        match (l_ty.as_ref(), r_ty.as_ref()) {
            (IntegerType { bits: l_bits }, IntegerType { bits: r_bits }) if *l_bits == *r_bits && *l_bits == $ret_bits => {
                match (l_bits, $sign) {
                    (0..64, false) => vec![
                        ExtactI64 { ret: $processor.get_tmp::<1>(), arg: $l_handler, pos: 0, len: $ret_bits },
                        ExtactI64 { ret: $processor.get_tmp::<2>(), arg: $r_handler, pos: 0, len: $ret_bits },
                        $op_64 { ret: $ret_handler, arg_1: $processor.get_tmp::<1>(), arg_2: $processor.get_tmp::<2>() },
                        ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                    ],
                    (0..64, true) => vec![
                        ShliI64 { ret: $processor.get_tmp::<1>(), arg_1: $l_handler, arg_2: 64 - ($ret_bits as i64) },
                        SariI64 { ret: $processor.get_tmp::<1>(), arg_1: $processor.get_tmp::<1>(), arg_2: 64 - ($ret_bits as i64) },
                        ShliI64 { ret: $processor.get_tmp::<2>(), arg_1: $r_handler, arg_2: 64 - ($ret_bits as i64) },
                        SariI64 { ret: $processor.get_tmp::<2>(), arg_1: $processor.get_tmp::<2>(), arg_2: 64 - ($ret_bits as i64) },
                        $op_64 { ret: $ret_handler, arg_1: $processor.get_tmp::<1>(), arg_2: $processor.get_tmp::<2>() },
                        ShliI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                        SariI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                    ],
                    (64, _) => vec![$op_64 { ret: $ret_handler, arg_1: $l_handler, arg_2: $r_handler }],
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
     $op_64_imm: ident,
     $sign: literal) => {{
        let v_ty = $processor.symbol_table.borrow().get(&$v_handler).unwrap().clone();
        match (v_ty.as_ref(), $constant.as_ref()) {
            (IntegerType { bits: v_bits }, Constant::Int { bits: c_bits, value })
                if *v_bits == *c_bits && *v_bits == $ret_bits =>
            {
                match (v_bits, $sign) {
                    (0..64, false) => vec![
                        ShliI64 { ret: $processor.get_tmp::<1>(), arg_1: $v_handler, arg_2: 64 - ($ret_bits as i64) },
                        SariI64 { ret: $processor.get_tmp::<1>(), arg_1: $processor.get_tmp::<1>(), arg_2: 64 - ($ret_bits as i64) },
                        $op_64_imm { ret: $ret_handler, arg_1: $processor.get_tmp::<1>(), arg_2: sign_extend_const(*value, $ret_bits) },
                        ShliI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                        SariI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                    ],
                    (0..64, true) => vec![
                        ExtactI64 { ret: $processor.get_tmp::<1>(), arg: $v_handler, pos: 0, len: $ret_bits },
                        $op_64_imm { ret: $ret_handler, arg_1: $processor.get_tmp::<1>(), arg_2: *value as i64 },
                        ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                    ],
                    (64, _) => vec![$op_64_imm { ret: $ret_handler, arg_1: $v_handler, arg_2: *value as i64 }],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

macro_rules! const_variable {
    ($processor: expr, $v_handler: expr, $constant: expr, $ret_hdler: expr, $ret_bits: expr,
     $op_64: ident,
     $sign: literal, $const_first: literal) => {{
        let v_ty = $processor.symbol_table.borrow().get(&$v_handler).unwrap().clone();
        match (v_ty.as_ref(), $constant.as_ref()) {
            (IntegerType { bits: v_bits }, Constant::Int { bits: c_bits, value })
                if *v_bits == *c_bits && *v_bits == $ret_bits =>
            {
                let mut ret = vec![MoviI64 { ret: $ret_hdler, arg: sign_extend_const(*value, $ret_bits) }];
                if $const_first {
                    ret.extend(two_variables!($processor, $ret_hdler, $v_handler, $ret_hdler, $ret_bits, $op_64, $sign));
                } else {
                    ret.extend(two_variables!($processor, $v_handler, $ret_hdler, $ret_hdler, $ret_bits, $op_64, $sign));
                }
                ret
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
                match (l_bits, $sign) {
                    (0..64, false) => vec![MoviI64 { ret: $ret_handler, arg: extract_const_u64(l_value $op r_value, $ret_bits) }],
                    (0..64, true) => vec![MoviI64 {
                        ret: $ret_handler,
                        arg: sign_extend_const(
                            (sign_extend_const(*l_value, $ret_bits) $op sign_extend_const(*r_value, $ret_bits)) as u64,
                            $ret_bits,
                        ),
                    }],
                    (64, _) => vec![MoviI64 { ret: $ret_handler, arg: (l_value $op r_value) as i64 }],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

macro_rules! int_op_impl {
    ($processor: expr, $inst: expr, $op: tt,
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
                two_variables!($processor, l_handler, r_handler, ret_handler, ret_bits, $op_64, $sign)
            }
            (LocalOperand { name: v_name, ty: _ }, ConstantOperand(constant)) => {
                let v_handler = $processor.name_to_handler(v_name);
                if $imm_tcg {
                    vc_imm_tcg!($processor, v_handler, constant, ret_handler, ret_bits, $op_64_imm, $sign)
                } else {
                    const_variable!($processor, v_handler, constant, ret_handler, ret_bits, $op_64, $sign, false)
                }
            }
            (ConstantOperand(constant), LocalOperand { name: v_name, ty: _ }) => {
                let v_handler = $processor.name_to_handler(v_name);
                if $imm_tcg {
                    vc_imm_tcg!($processor, v_handler, constant, ret_handler, ret_bits, $op_64_imm_2, $sign)
                } else {
                    const_variable!($processor, v_handler, constant, ret_handler, ret_bits, $op_64, $sign, true)
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
    ($processor: expr, $inst: expr, $op: tt, $op_64: ident, $sign: literal) => {
        int_op_impl!($processor, $inst, $op, $op_64, AddiI64, AddiI64, $sign, false)
    };
    ($processor: expr, $inst: expr, $op: tt,
     $op_64: ident, $op_64_imm: ident) => {
        int_op_impl!($processor, $inst, $op, $op_64, $op_64_imm, $op_64_imm, false, true)
    };
    ($processor: expr, $inst: expr, $op: tt,
     $op_64: ident, $op_64_imm: ident, $op_64_imm_2: ident) => {
        int_op_impl!($processor, $inst, $op, $op_64, $op_64_imm, $op_64_imm_2, false, true)
    };
}

impl Processor<'_> {
    pub fn add(&self, add: &Add) -> Vec<Tcg> {
        int_op!(self, add, +, AddI64, AddiI64)
    }

    pub fn sub(&self, sub: &Sub) -> Vec<Tcg> {
        int_op!(self, sub, -, SubI64, SubiI64, SubfiI64)
    }

    pub fn mul(&self, mul: &Mul) -> Vec<Tcg> {
        int_op!(self, mul, *, MulI64, MuliI64)
    }

    pub fn sdiv(&self, sdiv: &SDiv) -> Vec<Tcg> {
        int_op!(self, sdiv, /, DivI64, true)
    }

    pub fn udiv(&self, udiv: &UDiv) -> Vec<Tcg> {
        int_op!(self, udiv, /, DivuI64, false)
    }

    pub fn srem(&self, srem: &SRem) -> Vec<Tcg> {
        int_op!(self, srem, /, RemI64, true)
    }

    pub fn urem(&self, urem: &URem) -> Vec<Tcg> {
        int_op!(self, urem, /, RemuI64, false)
    }

    pub fn and(&self, and: &And) -> Vec<Tcg> {
        int_op!(self, and, &, AndI64, AndiI64)
    }

    pub fn or(&self, or: &Or) -> Vec<Tcg> {
        int_op!(self, or, |, OrI64, OriI64)
    }

    pub fn xor(&self, xor: &Xor) -> Vec<Tcg> {
        int_op!(self, xor, ^, XorI64, XoriI64)
    }

    pub fn shl(&self, shl: &Shl) -> Vec<Tcg> {
        int_op!(self, shl, <<, ShlI64, false)
    }

    pub fn shr(&self, shl: &LShr) -> Vec<Tcg> {
        int_op!(self, shl, >>, ShrI64, false)
    }

    pub fn sar(&self, shl: &AShr) -> Vec<Tcg> {
        int_op!(self, shl, >>, SarI64, true)
    }
}
