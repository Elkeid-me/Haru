use super::Processor;
use super::int_op::{extract_const_i64, sign_extend_const};
use super::tcg::Tcg::{self, *};
use llvm_ir::instruction::{FPExt, FPToSI, FPToUI, FPTrunc, SExt, SIToFP, Trunc, UIToFP, UnaryOp, ZExt};
use llvm_ir::types::Types;
use llvm_ir::{Constant, constant::Float, types::FPType};
use llvm_ir::{Operand::*, Type::*, instruction::HasResult};

impl Processor<'_> {
    pub fn fp_to_si(&self, fp_to_si: &FPToSI) -> Vec<Tcg> {
        let r_handler = self.name_to_handler(fp_to_si.get_result());
        let r_ty = self.symbol_table.borrow().get(&r_handler).unwrap().clone();
        let tmp_handler = self.new_handler();
        match fp_to_si.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), r_ty.as_ref()) {
                    (FPType(FPType::Double), IntegerType { bits }) if matches!(bits, 0..32) => {
                        self.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&self.module.types));
                        vec![
                            FcvtWD { ret: tmp_handler, arg: v_handler },
                            ExtrlI64I32 { ret: r_handler, arg: tmp_handler },
                            ShliI32 { ret: r_handler, arg_1: r_handler, arg_2: (32 - bits) as i32 },
                            SariI32 { ret: r_handler, arg_1: r_handler, arg_2: (32 - bits) as i32 },
                        ]
                    }
                    (FPType(FPType::Double), IntegerType { bits: 32 }) => {
                        self.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&self.module.types));
                        vec![FcvtWD { ret: tmp_handler, arg: v_handler }, ExtrlI64I32 { ret: r_handler, arg: tmp_handler }]
                    }
                    (FPType(FPType::Double), IntegerType { bits }) if matches!(bits, 33..64) => vec![
                        FcvtLD { ret: r_handler, arg: v_handler },
                        ShliI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                    ],
                    (FPType(FPType::Double), IntegerType { bits: 64 }) => vec![FcvtLD { ret: r_handler, arg: v_handler }],
                    (FPType(FPType::Single), IntegerType { bits }) if matches!(bits, 0..32) => {
                        self.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&self.module.types));
                        vec![
                            FcvtWS { ret: tmp_handler, arg: v_handler },
                            ExtrlI64I32 { ret: r_handler, arg: tmp_handler },
                            ShliI32 { ret: r_handler, arg_1: r_handler, arg_2: (32 - bits) as i32 },
                            SariI32 { ret: r_handler, arg_1: r_handler, arg_2: (32 - bits) as i32 },
                        ]
                    }
                    (FPType(FPType::Single), IntegerType { bits: 32 }) => {
                        self.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&self.module.types));
                        vec![FcvtWS { ret: tmp_handler, arg: v_handler }, ExtrlI64I32 { ret: r_handler, arg: tmp_handler }]
                    }
                    (FPType(FPType::Single), IntegerType { bits }) if matches!(bits, 33..64) => vec![
                        FcvtLS { ret: r_handler, arg: v_handler },
                        ShliI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                    ],
                    (FPType(FPType::Single), IntegerType { bits: 64 }) => vec![FcvtLS { ret: r_handler, arg: v_handler }],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Float(Float::Double(double)) => match r_ty.as_ref() {
                    IntegerType { bits } if matches!(bits, 0..=32) => vec![MoviI32 {
                        ret: r_handler,
                        arg: sign_extend_const(extract_const_i64(*double as i64, *bits) as u64, *bits) as i32,
                    }],
                    IntegerType { bits } if matches!(bits, 33..=64) => vec![MoviI64 {
                        ret: r_handler,
                        arg: sign_extend_const(extract_const_i64(*double as i64, *bits) as u64, *bits),
                    }],
                    _ => todo!(),
                },
                Constant::Float(Float::Single(single)) => match r_ty.as_ref() {
                    IntegerType { bits } if matches!(bits, 0..=32) => vec![MoviI32 {
                        ret: r_handler,
                        arg: sign_extend_const(extract_const_i64(*single as i64, *bits) as u64, *bits) as i32,
                    }],
                    IntegerType { bits } if matches!(bits, 33..=64) => vec![MoviI64 {
                        ret: r_handler,
                        arg: sign_extend_const(extract_const_i64(*single as i64, *bits) as u64, *bits),
                    }],
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn fp_to_ui(&self, fp_to_ui: &FPToUI) -> Vec<Tcg> {
        let r_handler = self.name_to_handler(fp_to_ui.get_result());
        let r_ty = self.symbol_table.borrow().get(&r_handler).unwrap().clone();
        let tmp_handler = self.new_handler();
        match fp_to_ui.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), r_ty.as_ref()) {
                    (FPType(FPType::Double), IntegerType { bits }) if matches!(bits, 0..32) => {
                        self.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&self.module.types));
                        vec![
                            FcvtWuD { ret: tmp_handler, arg: v_handler },
                            ExtrlI64I32 { ret: r_handler, arg: tmp_handler },
                            ExtactI32 { ret: r_handler, arg: r_handler, pos: 0, len: *bits },
                        ]
                    }
                    (FPType(FPType::Double), IntegerType { bits: 32 }) => {
                        self.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&self.module.types));
                        vec![FcvtWuD { ret: tmp_handler, arg: v_handler }, ExtrlI64I32 { ret: r_handler, arg: tmp_handler }]
                    }
                    (FPType(FPType::Double), IntegerType { bits }) if matches!(bits, 33..=64) => vec![
                        FcvtLuD { ret: r_handler, arg: v_handler },
                        ExtactI64 { ret: r_handler, arg: r_handler, pos: 0, len: *bits },
                    ],
                    (FPType(FPType::Single), IntegerType { bits }) if matches!(bits, 0..32) => {
                        self.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&self.module.types));
                        vec![
                            FcvtWuS { ret: tmp_handler, arg: v_handler },
                            ExtrlI64I32 { ret: r_handler, arg: tmp_handler },
                            ExtactI32 { ret: r_handler, arg: r_handler, pos: 0, len: *bits },
                        ]
                    }
                    (FPType(FPType::Single), IntegerType { bits: 32 }) => {
                        self.symbol_table.borrow_mut().insert(tmp_handler, Types::i64(&self.module.types));
                        vec![FcvtWuS { ret: tmp_handler, arg: v_handler }, ExtrlI64I32 { ret: r_handler, arg: tmp_handler }]
                    }
                    (FPType(FPType::Single), IntegerType { bits }) if matches!(bits, 33..64) => vec![
                        FcvtLuS { ret: r_handler, arg: v_handler },
                        ExtactI64 { ret: r_handler, arg: r_handler, pos: 0, len: *bits },
                    ],
                    (FPType(FPType::Single), IntegerType { bits: 64 }) => vec![FcvtLuS { ret: r_handler, arg: v_handler }],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Float(Float::Double(double)) => match r_ty.as_ref() {
                    IntegerType { bits } if matches!(bits, 0..=32) => {
                        vec![MoviI32 { ret: r_handler, arg: extract_const_i64(*double as i64, *bits) as i32 }]
                    }
                    IntegerType { bits } if matches!(bits, 33..=64) => {
                        vec![MoviI64 { ret: r_handler, arg: extract_const_i64(*double as i64, *bits) }]
                    }
                    _ => todo!(),
                },
                Constant::Float(Float::Single(single)) => match r_ty.as_ref() {
                    IntegerType { bits } if matches!(bits, 0..=32) => {
                        vec![MoviI32 { ret: r_handler, arg: extract_const_i64(*single as i64, *bits) as i32 }]
                    }
                    IntegerType { bits } if matches!(bits, 33..=64) => {
                        vec![MoviI64 { ret: r_handler, arg: extract_const_i64(*single as i64, *bits) }]
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn si_to_fp(&self, si_to_fp: &SIToFP) -> Vec<Tcg> {
        let r_handler = self.name_to_handler(si_to_fp.get_result());
        let r_ty = self.symbol_table.borrow().get(&r_handler).unwrap().clone();
        match si_to_fp.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), r_ty.as_ref()) {
                    (IntegerType { bits }, FPType(FPType::Double)) if matches!(bits, 0..32) => vec![
                        ExtuI32I64 { ret: r_handler, arg: v_handler },
                        ShliI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        FcvtDW { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits }, FPType(FPType::Single)) if matches!(bits, 0..32) => vec![
                        ExtuI32I64 { ret: r_handler, arg: v_handler },
                        ShliI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        FcvtSW { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits: 32 }, FPType(FPType::Double)) => {
                        vec![ExtuI32I64 { ret: r_handler, arg: v_handler }, FcvtDW { ret: r_handler, arg: r_handler }]
                    }
                    (IntegerType { bits: 32 }, FPType(FPType::Single)) => {
                        vec![ExtuI32I64 { ret: r_handler, arg: v_handler }, FcvtSW { ret: r_handler, arg: r_handler }]
                    }
                    (IntegerType { bits }, FPType(FPType::Double)) if matches!(bits, 33..64) => vec![
                        ShliI64 { ret: r_handler, arg_1: v_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        FcvtDL { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits }, FPType(FPType::Single)) if matches!(bits, 33..64) => vec![
                        ShliI64 { ret: r_handler, arg_1: v_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        FcvtSL { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits: 64 }, FPType(FPType::Double)) => vec![FcvtDL { ret: r_handler, arg: v_handler }],
                    (IntegerType { bits: 64 }, FPType(FPType::Single)) => vec![FcvtSL { ret: r_handler, arg: v_handler }],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Int { bits: 0..=64, value } => match r_ty.as_ref() {
                    FPType(FPType::Double) => vec![MoviI64 { ret: r_handler, arg: (*value as i64 as f64).to_bits() as i64 }],
                    FPType(FPType::Single) => vec![MoviI64 { ret: r_handler, arg: (*value as i64 as f32).to_bits() as i64 }],
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn ui_to_fp(&self, ui_to_fp: &UIToFP) -> Vec<Tcg> {
        let r_handler = self.name_to_handler(ui_to_fp.get_result());
        let binding = self.symbol_table.borrow();
        let r_ty = binding.get(&r_handler).unwrap().as_ref();
        match ui_to_fp.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), r_ty) {
                    (IntegerType { bits: 0..=32 }, FPType(FPType::Double)) => {
                        vec![ExtuI32I64 { ret: r_handler, arg: v_handler }, FcvtDWu { ret: r_handler, arg: r_handler }]
                    }
                    (IntegerType { bits: 0..=32 }, FPType(FPType::Single)) => {
                        vec![ExtuI32I64 { ret: r_handler, arg: v_handler }, FcvtSWu { ret: r_handler, arg: r_handler }]
                    }
                    (IntegerType { bits: 33..=64 }, FPType(FPType::Double)) => vec![FcvtDLu { ret: r_handler, arg: v_handler }],
                    (IntegerType { bits: 33..=64 }, FPType(FPType::Single)) => vec![FcvtSLu { ret: r_handler, arg: v_handler }],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Int { bits: 0..=64, value } => match r_ty {
                    FPType(FPType::Double) => vec![MoviI64 { ret: r_handler, arg: (*value as f64).to_bits() as i64 }],
                    FPType(FPType::Single) => vec![MoviI64 { ret: r_handler, arg: (*value as f32).to_bits() as i64 }],
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn trunc(&self, trunc: &Trunc) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(trunc.get_result());
        let ret_bits = match self.symbol_table.borrow().get(&ret_handler).unwrap().as_ref() {
            IntegerType { bits } => *bits,
            _ => todo!(),
        };
        match trunc.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), ret_bits) {
                    (IntegerType { bits: 0..=32 }, 0..=32) => {
                        vec![ExtactI32 { ret: ret_handler, arg: v_handler, pos: 0, len: ret_bits }]
                    }
                    (IntegerType { bits: 33..=64 }, 0..=32) => vec![
                        ExtrlI64I32 { ret: ret_handler, arg: v_handler },
                        ExtactI32 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                    ],
                    (IntegerType { bits: 33..=64 }, 33..=64) => {
                        vec![ExtactI64 { ret: ret_handler, arg: v_handler, pos: 0, len: ret_bits }]
                    }
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match (constant.as_ref(), ret_bits) {
                (Constant::Int { bits: _, value }, 0..=32) => {
                    vec![MoviI32 { ret: ret_handler, arg: (value & (u64::MAX >> (64 - ret_bits))) as i32 }]
                }
                (Constant::Int { bits: _, value }, 33..=64) => {
                    vec![MoviI64 { ret: ret_handler, arg: (value & (u64::MAX >> (64 - ret_bits))) as i64 }]
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn zext(&self, zext: &ZExt) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(zext.get_result());
        let ret_bits = match self.symbol_table.borrow().get(&ret_handler).unwrap().as_ref() {
            IntegerType { bits } => *bits,
            _ => todo!(),
        };
        match zext.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), ret_bits) {
                    (IntegerType { bits }, 0..=32) if matches!(bits, 0..=32) => {
                        vec![MovI32 { ret: ret_handler, arg: v_handler }]
                    }
                    (IntegerType { bits }, 33..=64) if matches!(bits, 0..=32) => {
                        vec![ExtuI32I64 { ret: ret_handler, arg: v_handler }]
                    }
                    (IntegerType { bits }, 33..=64) if matches!(bits, 33..=64) => {
                        vec![MovI64 { ret: ret_handler, arg: v_handler }]
                    }
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match (constant.as_ref(), ret_bits) {
                (Constant::Int { bits, value }, 0..=32) => {
                    vec![MoviI32 { ret: ret_handler, arg: (value & (u64::MAX >> (64 - bits))) as i32 }]
                }
                (Constant::Int { bits, value }, 33..=64) => {
                    vec![MoviI64 { ret: ret_handler, arg: (value & (u64::MAX >> (64 - bits))) as i64 }]
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn sext(&self, sext: &SExt) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(sext.get_result());
        let ret_bits = match self.symbol_table.borrow().get(&ret_handler).unwrap().as_ref() {
            IntegerType { bits } => *bits,
            _ => todo!(),
        };
        match sext.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), ret_bits) {
                    (IntegerType { bits }, 0..=32) if matches!(bits, 0..32) => vec![
                        ShliI32 { ret: ret_handler, arg_1: ret_handler, arg_2: (32 - bits) as i32 },
                        SariI32 { ret: ret_handler, arg_1: ret_handler, arg_2: (32 - bits) as i32 },
                    ],
                    (IntegerType { bits }, 33..=64) if matches!(bits, 0..=32) => vec![
                        ExtuI32I64 { ret: ret_handler, arg: v_handler },
                        ShliI64 { ret: ret_handler, arg_1: ret_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: ret_handler, arg_1: ret_handler, arg_2: (64 - bits) as i64 },
                    ],
                    (IntegerType { bits }, 33..=64) if matches!(bits, 33..64) => vec![
                        ShliI64 { ret: ret_handler, arg_1: ret_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: ret_handler, arg_1: ret_handler, arg_2: (64 - bits) as i64 },
                    ],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match (constant.as_ref(), ret_bits) {
                (Constant::Int { bits, value }, 0..=32) => {
                    vec![MoviI32 { ret: ret_handler, arg: (*value as i32) << (32 - *bits) >> (32 - *bits) }]
                }
                (Constant::Int { bits, value }, 33..=64) => {
                    vec![MoviI64 { ret: ret_handler, arg: (*value as i64) << (64 - *bits) >> (64 - *bits) }]
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn fp_ext(&self, fp_ext: &FPExt) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(fp_ext.get_result());
        let binding = self.symbol_table.borrow();
        let r_ty = binding.get(&ret_handler).unwrap().as_ref();
        match fp_ext.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), r_ty) {
                    (FPType(FPType::Single), FPType(FPType::Double)) => vec![FcvtDS { ret: ret_handler, arg: v_handler }],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match (constant.as_ref(), r_ty) {
                (Constant::Float(Float::Single(single)), FPType(FPType::Double)) => {
                    vec![MoviI64 { ret: ret_handler, arg: (*single as f64).to_bits() as i64 }]
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn fp_trunc(&self, fp_trunc: &FPTrunc) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(fp_trunc.get_result());
        let binding = self.symbol_table.borrow();
        let r_ty = binding.get(&ret_handler).unwrap().as_ref();
        match fp_trunc.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), r_ty) {
                    (FPType(FPType::Double), FPType(FPType::Single)) => vec![FcvtSD { ret: ret_handler, arg: v_handler }],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match (constant.as_ref(), r_ty) {
                (Constant::Float(Float::Double(double)), FPType(FPType::Single)) => {
                    vec![MoviI64 { ret: ret_handler, arg: (*double as f32).to_bits() as i64 }]
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}
