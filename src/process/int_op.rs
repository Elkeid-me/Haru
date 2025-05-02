use super::Processor;
use super::tcg::Tcg::{self, *};
use llvm_ir::Constant;
use llvm_ir::instruction::{AShr, Add, And, ICmp, LShr, Mul, Or, SDiv, SRem, Shl, Sub, UDiv, URem, Xor};
use llvm_ir::{Operand::*, Type::*, instruction::HasResult};

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
     $sign: literal, $is_bool: literal) => {{
        let l_ty = $processor.symbol_table.borrow().get(&$l_handler).unwrap().clone();
        let r_ty = $processor.symbol_table.borrow().get(&$r_handler).unwrap().clone();
        match (l_ty.as_ref(), r_ty.as_ref()) {
            (IntegerType { bits: l_bits }, IntegerType { bits: r_bits })
                if *l_bits == *r_bits && (*l_bits == $ret_bits || $is_bool && $ret_bits == 1) =>
            {
                match (l_bits, $sign) {
                    (0..64, false) => {
                        let tmp_1 = $processor.get_tmp::<1>();
                        let tmp_2 = $processor.get_tmp::<2>();
                        $processor.use_variable(tmp_1);
                        $processor.use_variable(tmp_2);
                        vec![
                            ExtactI64 { ret: tmp_1, arg: $l_handler, pos: 0, len: *l_bits },
                            ExtactI64 { ret: tmp_2, arg: $r_handler, pos: 0, len: *r_bits },
                            $op_64 { ret: $ret_handler, arg_1: tmp_1, arg_2: tmp_2 },
                            ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                        ]
                    }
                    (0..64, true) => {
                        let tmp_1 = $processor.get_tmp::<1>();
                        let tmp_2 = $processor.get_tmp::<2>();
                        $processor.use_variable(tmp_1);
                        $processor.use_variable(tmp_2);
                        vec![
                            ShliI64 { ret: tmp_1, arg_1: $l_handler, arg_2: 64 - (*l_bits as i64) },
                            SariI64 { ret: tmp_1, arg_1: tmp_1, arg_2: 64 - (*l_bits as i64) },
                            ShliI64 { ret: tmp_2, arg_1: $r_handler, arg_2: 64 - (*r_bits as i64) },
                            SariI64 { ret: tmp_2, arg_1: tmp_2, arg_2: 64 - (*r_bits as i64) },
                            $op_64 { ret: $ret_handler, arg_1: tmp_1, arg_2: tmp_2 },
                            ShliI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                            SariI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                        ]
                    }
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
                    (0..64, false) => {
                        let tmp_1 = $processor.get_tmp::<1>();
                        $processor.use_variable(tmp_1);
                        vec![
                            ShliI64 { ret: tmp_1, arg_1: $v_handler, arg_2: 64 - (*v_bits as i64) },
                            SariI64 { ret: tmp_1, arg_1: tmp_1, arg_2: 64 - (*v_bits as i64) },
                            $op_64_imm { ret: $ret_handler, arg_1: tmp_1, arg_2: sign_extend_const(*value, *c_bits) },
                            ShliI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                            SariI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                        ]
                    }
                    (0..64, true) => {
                        let tmp_1 = $processor.get_tmp::<1>();
                        $processor.use_variable(tmp_1);
                        vec![
                            ExtactI64 { ret: tmp_1, arg: $v_handler, pos: 0, len: *v_bits },
                            $op_64_imm { ret: $ret_handler, arg_1: tmp_1, arg_2: *value as i64 },
                            ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                        ]
                    }
                    (64, _) => vec![$op_64_imm { ret: $ret_handler, arg_1: $v_handler, arg_2: *value as i64 }],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

macro_rules! cv_helper {
    ($const_first: literal, $op_64: ident, $ret_handler: expr, $const_arg: expr, $v_arg: expr) => {
        if $const_first {
            $op_64 { ret: $ret_handler, arg_1: $const_arg, arg_2: $v_arg }
        } else {
            $op_64 { ret: $ret_handler, arg_1: $v_arg, arg_2: $const_arg }
        }
    };
}

macro_rules! const_variable {
    ($processor: expr, $v_handler: expr, $constant: expr, $ret_handler: expr, $ret_bits: expr,
     $op_64: ident,
     $sign: literal, $const_first: literal, $is_bool: literal) => {{
        let v_ty = $processor.symbol_table.borrow().get(&$v_handler).unwrap().clone();
        match (v_ty.as_ref(), $constant.as_ref()) {
            (IntegerType { bits: v_bits }, Constant::Int { bits: c_bits, value })
                if *v_bits == *c_bits && (*v_bits == $ret_bits || $is_bool && $ret_bits == 1) =>
            {
                let tmp_1 = $processor.get_tmp::<1>();
                let tmp_2 = $processor.get_tmp::<2>();
                $processor.use_variable(tmp_1);
                match (v_bits, $sign) {
                    (0..64, false) => {
                        $processor.use_variable(tmp_2);
                        vec![
                            MoviI64 { ret: tmp_1, arg: *value as i64 },
                            ExtactI64 { ret: tmp_2, arg: $v_handler, pos: 0, len: *v_bits },
                            cv_helper!($const_first, $op_64, $ret_handler, tmp_1, tmp_2),
                            ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                        ]
                    }
                    (0..64, true) => {
                        $processor.use_variable(tmp_2);
                        vec![
                            MoviI64 { ret: tmp_1, arg: sign_extend_const(*value, *c_bits) },
                            ShliI64 { ret: tmp_2, arg_1: $v_handler, arg_2: 64 - (*v_bits as i64) },
                            SariI64 { ret: tmp_2, arg_1: tmp_2, arg_2: 64 - (*v_bits as i64) },
                            cv_helper!($const_first, $op_64, $ret_handler, tmp_1, tmp_2),
                            ShliI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                            SariI64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: 64 - ($ret_bits as i64) },
                        ]
                    }
                    (64, _) => vec![
                        MoviI64 { ret: tmp_1, arg: *value as i64 },
                        cv_helper!($const_first, $op_64, $ret_handler, tmp_1, $v_handler),
                    ],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

/// 处理两个 Const 操作数的情况
macro_rules! two_consts {
    ($op: tt, $l_constant: expr, $r_constant: expr, $ret_handler: expr, $ret_bits: expr, $sign: literal, $is_bool: literal) => {{
        match ($l_constant.as_ref(), $r_constant.as_ref()) {
            (Constant::Int { bits: l_bits, value: l_value }, Constant::Int { bits: r_bits, value: r_value })
                if *l_bits == *r_bits && (*l_bits == $ret_bits || $is_bool && $ret_bits == 1) =>
            {
                match (l_bits, $sign) {
                    (0..64, false) => vec![MoviI64 { ret: $ret_handler, arg: extract_const_u64((l_value $op r_value) as u64, $ret_bits) }],
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
     $sign: literal, $imm_tcg: literal, $is_bool: literal) => {{
        let ret_handler = $processor.name_to_handler($inst.get_result());
        let ret_bits = match $processor.symbol_table.borrow().get(&ret_handler).unwrap().as_ref() {
            IntegerType { bits } => *bits,
            _ => todo!(),
        };
        match (&$inst.operand0, &$inst.operand1) {
            (LocalOperand { name: l_name, ty: _ }, LocalOperand { name: r_name, ty: _ }) => {
                let l_handler = $processor.name_to_handler(l_name);
                let r_handler = $processor.name_to_handler(r_name);
                $processor.use_variable(l_handler);
                $processor.use_variable(r_handler);
                $processor.use_variable(ret_handler);
                two_variables!($processor, l_handler, r_handler, ret_handler, ret_bits, $op_64, $sign, $is_bool)
            }
            (LocalOperand { name: v_name, ty: _ }, ConstantOperand(constant)) => {
                let v_handler = $processor.name_to_handler(v_name);
                $processor.use_variable(v_handler);
                $processor.use_variable(ret_handler);
                if $imm_tcg {
                    vc_imm_tcg!($processor, v_handler, constant, ret_handler, ret_bits, $op_64_imm, $sign)
                } else {
                    const_variable!($processor, v_handler, constant, ret_handler, ret_bits, $op_64, $sign, false, $is_bool)
                }
            }
            (ConstantOperand(constant), LocalOperand { name: v_name, ty: _ }) => {
                let v_handler = $processor.name_to_handler(v_name);
                $processor.use_variable(v_handler);
                $processor.use_variable(ret_handler);
                if $imm_tcg {
                    vc_imm_tcg!($processor, v_handler, constant, ret_handler, ret_bits, $op_64_imm_2, $sign)
                } else {
                    const_variable!($processor, v_handler, constant, ret_handler, ret_bits, $op_64, $sign, true, $is_bool)
                }
            }
            (ConstantOperand(l_constant), ConstantOperand(r_constant)) => {
                $processor.use_variable(ret_handler);
                two_consts!($op, l_constant, r_constant, ret_handler, ret_bits, $sign, $is_bool)
            }
            _ => todo!(),
        }
    }};
}

macro_rules! int_op {
    ($processor: expr, $inst: expr, $op: tt, $op_64: ident, $sign: literal) => {
        int_op_impl!($processor, $inst, $op, $op_64, AddiI64, AddiI64, $sign, false, false)
    };
    ($processor: expr, $inst: expr, $op: tt, $op_64: ident, $sign: literal, $is_bool: literal) => {
        int_op_impl!($processor, $inst, $op, $op_64, AddiI64, AddiI64, $sign, false, $is_bool)
    };
    ($processor: expr, $inst: expr, $op: tt,
     $op_64: ident, $op_64_imm: ident) => {
        int_op_impl!($processor, $inst, $op, $op_64, $op_64_imm, $op_64_imm, false, true, false)
    };
    ($processor: expr, $inst: expr, $op: tt,
     $op_64: ident, $op_64_imm: ident, $op_64_imm_2: ident) => {
        int_op_impl!($processor, $inst, $op, $op_64, $op_64_imm, $op_64_imm_2, false, true, false)
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
    pub fn icmp(&self, icmp: &ICmp) -> Vec<Tcg> {
        match icmp.predicate {
            llvm_ir::IntPredicate::EQ => int_op!(self, icmp, ==, SetcondEqI64, false, true),
            llvm_ir::IntPredicate::NE => int_op!(self, icmp, !=, SetcondNeI64, false, true),
            llvm_ir::IntPredicate::UGT => int_op!(self, icmp, >, SetcondUgtI64, false, true),
            llvm_ir::IntPredicate::UGE => int_op!(self, icmp, >=, SetcondUgeI64, false, true),
            llvm_ir::IntPredicate::ULT => int_op!(self, icmp, <, SetcondUltI64, false, true),
            llvm_ir::IntPredicate::ULE => int_op!(self, icmp, <=, SetcondUleI64, false, true),
            llvm_ir::IntPredicate::SGT => int_op!(self, icmp, >, SetcondSgtI64, true, true),
            llvm_ir::IntPredicate::SGE => int_op!(self, icmp, >=, SetcondSgeI64, true, true),
            llvm_ir::IntPredicate::SLT => int_op!(self, icmp, <, SetcondSltI64, true, true),
            llvm_ir::IntPredicate::SLE => int_op!(self, icmp, <=, SetcondSleI64, true, true),
        }
    }
}
