use super::Processor;
use super::tcg::Tcg::{self, *};
use llvm_ir::Constant;
use llvm_ir::instruction::{Add, And, Mul, Or, SDiv, SRem, Sub, UDiv, URem, Xor};
use llvm_ir::{Operand::*, Type::*, instruction::BinaryOp, instruction::HasResult};

/// 面善又友善的奇妙宏
/// 处理两个非 Const 操作数的情况
macro_rules! two_operand {
    ($processor: expr, $l_name: expr, $r_name: expr, $l_ty: expr, $r_ty: expr, $ret_handler: expr, $ret_bits: expr, $op_32: ident, $op_64: ident) => {{
        let l_handler = $processor.name_to_handler($l_name);
        let r_handler = $processor.name_to_handler($r_name);
        match ($l_ty.as_ref(), $r_ty.as_ref()) {
            (IntegerType { bits: l_bits }, IntegerType { bits: r_bits }) if *l_bits == *r_bits && *l_bits == $ret_bits => {
                match l_bits {
                    0..32 => vec![
                        $op_32 { ret: $ret_handler, arg_1: l_handler, arg_2: r_handler },
                        ExtactI32 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                    ],
                    32 => vec![$op_32 { ret: $ret_handler, arg_1: l_handler, arg_2: r_handler }],
                    33..64 => vec![
                        $op_64 { ret: $ret_handler, arg_1: l_handler, arg_2: r_handler },
                        ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                    ],
                    64 => vec![$op_64 { ret: $ret_handler, arg_1: l_handler, arg_2: r_handler }],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

/// 处理一个非 Const 操作数和一个 Const 操作数的情况
///
/// 可以配合 [`SubfiI32`] 等的参数反向食用
macro_rules! value_const {
    ($processor: expr, $v_name: expr, $v_ty: expr, $constant: expr, $ret_handler: expr, $ret_bits: expr, $op_32_imm: ident, $op_64_imm: ident) => {{
        let v_handler = $processor.name_to_handler($v_name);
        match ($v_ty.as_ref(), $constant.as_ref()) {
            (IntegerType { bits: v_bits }, Constant::Int { bits: c_bits, value })
                if *v_bits == *c_bits && *v_bits == $ret_bits =>
            {
                match v_bits {
                    0..32 => vec![
                        $op_32_imm { ret: $ret_handler, arg_1: v_handler, arg_2: *value as i32 },
                        ExtactI32 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                    ],
                    32 => vec![$op_32_imm { ret: $ret_handler, arg_1: v_handler, arg_2: *value as i32 }],
                    33..64 => vec![
                        $op_64_imm { ret: $ret_handler, arg_1: v_handler, arg_2: *value as i64 },
                        ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                    ],
                    64 => vec![$op_64_imm { ret: $ret_handler, arg_1: v_handler, arg_2: *value as i64 }],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

/// 处理两个 Const 操作数的情况
macro_rules! two_const {
    ($op: tt, $l_constant: expr, $r_constant: expr, $ret_handler: expr, $ret_bits: expr) => {{
        match ($l_constant.as_ref(), $r_constant.as_ref()) {
            (Constant::Int { bits: l_bits, value: l_value }, Constant::Int { bits: r_bits, value: r_value })
                if *l_bits == *r_bits && *l_bits == $ret_bits =>
            {
                match l_bits {
                    0..32 => vec![
                        MoviI32 { ret: $ret_handler, arg: (l_value $op r_value) as i32 },
                        ExtactI32 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                    ],
                    32 => vec![MoviI32 { ret: $ret_handler, arg: (l_value $op r_value) as i32 }],
                    33..64 => vec![
                        MoviI64 { ret: $ret_handler, arg: (l_value $op r_value) as i64 },
                        ExtactI64 { ret: $ret_handler, arg: $ret_handler, pos: 0, len: $ret_bits },
                    ],
                    64 => vec![MoviI64 { ret: $ret_handler, arg: (l_value $op r_value) as i64 }],
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }};
}

/// 可交换的整数算数
macro_rules! com_int_op {
    ($processor: expr, $inst: expr, $op: tt, $op_32: ident, $op_32_imm: ident, $op_64: ident, $op_64_imm: ident) => {{
        let ret_handler = $processor.name_to_handler($inst.get_result());
        let ret_bits = match *$processor.symbol_table.borrow().get(&ret_handler).unwrap().as_ref() {
            IntegerType { bits } => bits,
            _ => todo!(),
        };
        match ($inst.get_operand0(), $inst.get_operand1()) {
            (LocalOperand { name: l_name, ty: l_ty }, LocalOperand { name: r_name, ty: r_ty }) => {
                two_operand!($processor, l_name, r_name, l_ty, r_ty, ret_handler, ret_bits, $op_32, $op_64)
            }
            (LocalOperand { name: v_name, ty: v_ty }, ConstantOperand(constant))
            | (ConstantOperand(constant), LocalOperand { name: v_name, ty: v_ty }) => {
                value_const!($processor, v_name, v_ty, constant, ret_handler, ret_bits, $op_32_imm, $op_64_imm)
            }
            (ConstantOperand(l_constant), ConstantOperand(r_constant)) => {
                two_const!($op, l_constant, r_constant, ret_handler, ret_bits)
            }
            _ => todo!(),
        }
    }};
}

/// 不可交换的整数算数
macro_rules! noncom_int_op {
    ($processor: expr, $inst: expr, $op: tt, $op_32: ident, $op_32_imm: ident, $op_32_imm_f: ident, $op_64: ident, $op_64_imm: ident, $op_64_imm_f: ident) => {{
        let ret_handler = $processor.name_to_handler($inst.get_result());
        let ret_bits = match *$processor.symbol_table.borrow().get(&ret_handler).unwrap().as_ref() {
            IntegerType { bits } => bits,
            _ => todo!(),
        };
        match ($inst.get_operand0(), $inst.get_operand1()) {
            (LocalOperand { name: l_name, ty: l_ty }, LocalOperand { name: r_name, ty: r_ty }) => {
                two_operand!($processor, l_name, r_name, l_ty, r_ty, ret_handler, ret_bits, $op_32, $op_64)
            }
            (LocalOperand { name: v_name, ty: v_ty }, ConstantOperand(constant)) => {
                value_const!($processor, v_name, v_ty, constant, ret_handler, ret_bits, $op_32_imm, $op_64_imm)
            }
            (ConstantOperand(constant), LocalOperand { name: v_name, ty: v_ty }) => {
                value_const!($processor, v_name, v_ty, constant, ret_handler, ret_bits, $op_32_imm_f, $op_64_imm_f)
            }
            (ConstantOperand(l_constant), ConstantOperand(r_constant)) => {
                two_const!($op, l_constant, r_constant, ret_handler, ret_bits)
            }
            _ => todo!(),
        }
    }};
}

impl Processor<'_> {
    pub fn add(&self, add: &Add) -> Vec<Tcg> {
        com_int_op!(self, add, +, AddI32, AddiI32, AddI64, AddiI64)
    }

    pub fn sub(&self, sub: &Sub) -> Vec<Tcg> {
        noncom_int_op!(self, sub, -, SubI32, SubiI32, SubfiI32, SubI64, SubiI64, SubfiI64)
    }

    pub fn mul(&self, mul: &Mul) -> Vec<Tcg> {
        com_int_op!(self, mul, *, MulI32, MuliI32, MulI64, MuliI64)
    }

    pub fn sdiv(&self, sdiv: &SDiv) -> Vec<Tcg> {
        noncom_int_op!(self, sdiv, /, DivI32, DiviI32, DivfiI32, DivI64, DiviI64, DivfiI64)
    }

    pub fn udiv(&self, udiv: &UDiv) -> Vec<Tcg> {
        noncom_int_op!(self, udiv, /, DivuI32, DivuiI32, DivufiI32, DivuI64, DivuiI64, DivufiI64)
    }

    pub fn srem(&self, srem: &SRem) -> Vec<Tcg> {
        noncom_int_op!(self, srem, %, RemI32, RemiI32, RemfiI32, RemI64, RemiI64, RemfiI64)
    }

    pub fn urem(&self, urem: &URem) -> Vec<Tcg> {
        noncom_int_op!(self, urem, %, RemuI32, RemuiI32, RemufiI32, RemuI64, RemuiI64, RemufiI64)
    }

    pub fn and(&self, and: &And) -> Vec<Tcg> {
        com_int_op!(self, and, &, AndI32, AndiI32, AndI64, AndiI64)
    }

    pub fn or(&self, or: &Or) -> Vec<Tcg> {
        com_int_op!(self, or, |, OrI32, OriI32, OrI64, OriI64)
    }

    pub fn xor(&self, xor: &Xor) -> Vec<Tcg> {
        com_int_op!(self, xor, ^, XorI32, XoriI32, XorI64, XoriI64)
    }
}
