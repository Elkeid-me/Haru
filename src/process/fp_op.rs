use super::Processor;
use super::tcg::Tcg::{self, *};
use llvm_ir::instruction::{FAdd, FDiv, FMul, FNeg, FRem, FSub, UnaryOp};
use llvm_ir::{Constant, constant::Float, types::FPType};
use llvm_ir::{Operand::*, Type::*, instruction::BinaryOp, instruction::HasResult};

/// 处理两个非 Const 操作数
macro_rules! two_operand {
    ($processor: expr, $l_handler: expr, $r_handler: expr, $ret_handler: expr, $op_32: ident, $op_64: ident) => {{
        let arg_ty = $processor.symbol_table.borrow().get(&$l_handler).unwrap().clone();
        match arg_ty.as_ref() {
            FPType(FPType::Double) => vec![$op_64 { ret: $ret_handler, arg_1: $l_handler, arg_2: $r_handler }],
            FPType(FPType::Single) => vec![$op_32 { ret: $ret_handler, arg_1: $l_handler, arg_2: $r_handler }],
            _ => todo!(),
        }
    }};
}

/// 处理第一个为非 Const 操作数、第二个为 Const 操作数的情况
macro_rules! value_const {
    ($processor: expr, $v_handler: expr, $constant: expr, $ret_handler: expr, $op_32: ident, $op_64: ident) => {{
        let v_ty = $processor.symbol_table.borrow().get(&$v_handler).unwrap().clone();
        match (v_ty.as_ref(), $constant.as_ref()) {
            (FPType(FPType::Double), Constant::Float(Float::Double(double))) => vec![
                MoviI64 { ret: $ret_handler, arg: double.to_bits() as i64 },
                $op_64 { ret: $ret_handler, arg_1: $v_handler, arg_2: $ret_handler },
            ],
            (FPType(FPType::Single), Constant::Float(Float::Single(single))) => vec![
                MoviI64 { ret: $ret_handler, arg: (single.to_bits() as u64 | 0xffff_ffff_0000_0000) as i64 },
                $op_32 { ret: $ret_handler, arg_1: $v_handler, arg_2: $ret_handler },
            ],
            _ => todo!(),
        }
    }};
}

/// 处理第一个为 Const 操作数、第二个非 Const 操作数的情况
macro_rules! const_value {
    ($processor: expr, $v_handler: expr, $constant: expr, $ret_handler: expr, $op_32: ident, $op_64: ident) => {{
        let v_ty = $processor.symbol_table.borrow().get(&$v_handler).unwrap().clone();
        match ($constant.as_ref(), v_ty.as_ref()) {
            (Constant::Float(Float::Double(double)), FPType(FPType::Double)) => vec![
                MoviI64 { ret: $ret_handler, arg: double.to_bits() as i64 },
                $op_64 { ret: $ret_handler, arg_1: $ret_handler, arg_2: $v_handler },
            ],
            (Constant::Float(Float::Single(single)), FPType(FPType::Single)) => vec![
                MoviI64 { ret: $ret_handler, arg: (single.to_bits() as u64 | 0xffff_ffff_0000_0000) as i64 },
                $op_32 { ret: $ret_handler, arg_1: $ret_handler, arg_2: $v_handler },
            ],
            _ => todo!(),
        }
    }};
}

/// 处理两个 Const 操作数
macro_rules! two_const {
    ($op: tt, $l_constant: expr, $r_constant: expr, $ret_handler: expr) => {{
        match ($l_constant.as_ref(), $r_constant.as_ref()) {
            (
                Constant::Float(Float::Double(l)),
                Constant::Float(Float::Double(r)),
            ) => vec![MoviI64 { ret: $ret_handler, arg: (l $op r).to_bits() as i64 }],
            (
                Constant::Float(Float::Single(l)),
                Constant::Float(Float::Single(r)),
            ) => vec![MoviI64 { ret: $ret_handler, arg: ((l $op r).to_bits() as u64 | 0xffff_ffff_0000_0000) as i64 }],
            _ => todo!(),
        }}
    };
}

macro_rules! fp_op {
    ($processor: expr, $inst: expr, $op: tt, $op_32: ident, $op_64: ident) => {{
        let ret_handler = $processor.name_to_handler($inst.get_result());
        match ($inst.get_operand0(), $inst.get_operand1()) {
            (LocalOperand { name: l_name, ty: _ }, LocalOperand { name: r_name, ty: _ }) => {
                let l_handler = $processor.name_to_handler(l_name);
                let r_handler = $processor.name_to_handler(r_name);
                $processor.use_variable(l_handler);
                $processor.use_variable(r_handler);
                $processor.use_variable(ret_handler);
                two_operand!($processor, l_handler, r_handler, ret_handler, $op_32, $op_64)
            }
            (LocalOperand { name: v_name, ty: _ }, ConstantOperand(constant)) => {
                let v_handler = $processor.name_to_handler(v_name);
                $processor.use_variable(v_handler);
                $processor.use_variable(ret_handler);
                value_const!($processor, v_handler, constant, ret_handler, $op_32, $op_64)
            }
            (ConstantOperand(constant), LocalOperand { name: v_name, ty: _ }) => {
                let v_handler = $processor.name_to_handler(v_name);
                $processor.use_variable(v_handler);
                $processor.use_variable(ret_handler);
                const_value!($processor, v_handler, constant, ret_handler, $op_32, $op_64)
            }
            (ConstantOperand(l_constant), ConstantOperand(r_constant)) => {
                $processor.use_variable(ret_handler);
                two_const!($op, l_constant, r_constant, ret_handler)
            }
            _ => todo!(),
        }
    }};
}

impl Processor<'_> {
    pub fn fadd(&self, fadd: &FAdd) -> Vec<Tcg> {
        fp_op!(self, fadd, +, FaddS, FaddD)
    }

    pub fn fsub(&self, fsub: &FSub) -> Vec<Tcg> {
        fp_op!(self, fsub, -, FsubS, FsubD)
    }

    pub fn fmul(&self, fmul: &FMul) -> Vec<Tcg> {
        fp_op!(self, fmul, +, FmulS, FmulD)
    }

    pub fn fdiv(&self, fdiv: &FDiv) -> Vec<Tcg> {
        fp_op!(self, fdiv, /, FdivS, FdivD)
    }

    pub fn frem(&self, _frem: &FRem) -> Vec<Tcg> {
        todo!()
    }

    pub fn fneg(&self, fneg: &FNeg) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(fneg.get_result());
        self.use_variable(ret_handler);
        match &fneg.get_operand() {
            LocalOperand { name, ty } => {
                let arg_handler = self.name_to_handler(name);
                self.use_variable(arg_handler);
                match ty.as_ref() {
                    FPType(FPType::Double) => vec![XoriI64 { ret: ret_handler, arg_1: arg_handler, arg_2: 1i64 << 63 }],
                    FPType(FPType::Single) => vec![XoriI64 { ret: ret_handler, arg_1: arg_handler, arg_2: 1i64 << 31 }],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Float(Float::Double(double)) => vec![MoviI64 { ret: ret_handler, arg: (-double).to_bits() as i64 }],
                Constant::Float(Float::Single(single)) => vec![MoviI64 { ret: ret_handler, arg: (-single).to_bits() as i64 }],
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}
