use super::Processor;
use super::tcg::Tcg::{self, *};
use llvm_ir::instruction::{FPToSI, FPToUI, SIToFP, UIToFP, UnaryOp};
use llvm_ir::types::Typed;
use llvm_ir::{Constant, constant::Float, types::FPType};
use llvm_ir::{Operand::*, Type::*, instruction::HasResult};

impl Processor<'_> {
    pub fn fp_to_si(&self, fp_to_si: &FPToSI) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(fp_to_si.get_result());
        match fp_to_si.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match ty.as_ref() {
                    FPType(FPType::Double) => match fp_to_si.get_type(&self.module.types).as_ref() {
                        IntegerType { bits: 0..=32 } => vec![FcvtWD { ret: ret_handler, arg: v_handler }],
                        IntegerType { bits: 33..=64 } => vec![FcvtLD { ret: ret_handler, arg: v_handler }],
                        _ => todo!(),
                    },
                    FPType(FPType::Single) => match fp_to_si.get_type(&self.module.types).as_ref() {
                        IntegerType { bits: 0..=32 } => vec![FcvtWS { ret: ret_handler, arg: v_handler }],
                        IntegerType { bits: 33..=64 } => vec![FcvtLS { ret: ret_handler, arg: v_handler }],
                        _ => todo!(),
                    },
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Float(Float::Double(double)) => match fp_to_si.get_type(&self.module.types).as_ref() {
                    IntegerType { bits: 0..=32 } => vec![MoviI32 { ret: ret_handler, arg: *double as i32 }],
                    IntegerType { bits: 33..=64 } => vec![MoviI64 { ret: ret_handler, arg: *double as i64 }],
                    _ => todo!(),
                },
                Constant::Float(Float::Single(single)) => match fp_to_si.get_type(&self.module.types).as_ref() {
                    IntegerType { bits: 0..=32 } => vec![MoviI32 { ret: ret_handler, arg: *single as i32 }],
                    IntegerType { bits: 33..=64 } => vec![MoviI64 { ret: ret_handler, arg: *single as i64 }],
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn fp_to_ui(&self, fp_to_ui: &FPToUI) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(fp_to_ui.get_result());
        match fp_to_ui.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match ty.as_ref() {
                    FPType(FPType::Double) => match fp_to_ui.get_type(&self.module.types).as_ref() {
                        IntegerType { bits: 0..=32 } => vec![FcvtWuD { ret: ret_handler, arg: v_handler }],
                        IntegerType { bits: 33..=64 } => vec![FcvtLuD { ret: ret_handler, arg: v_handler }],
                        _ => todo!(),
                    },
                    FPType(FPType::Single) => match fp_to_ui.get_type(&self.module.types).as_ref() {
                        IntegerType { bits: 0..=32 } => vec![FcvtWuS { ret: ret_handler, arg: v_handler }],
                        IntegerType { bits: 33..=64 } => vec![FcvtLuS { ret: ret_handler, arg: v_handler }],
                        _ => todo!(),
                    },
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Float(Float::Double(double)) => match fp_to_ui.get_type(&self.module.types).as_ref() {
                    IntegerType { bits: 0..=32 } => vec![MoviI32 { ret: ret_handler, arg: *double as i32 }],
                    IntegerType { bits: 33..=64 } => vec![MoviI64 { ret: ret_handler, arg: *double as i64 }],
                    _ => todo!(),
                },
                Constant::Float(Float::Single(single)) => match fp_to_ui.get_type(&self.module.types).as_ref() {
                    IntegerType { bits: 0..=32 } => vec![MoviI32 { ret: ret_handler, arg: *single as i32 }],
                    IntegerType { bits: 33..=64 } => vec![MoviI64 { ret: ret_handler, arg: *single as i64 }],
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn si_to_fp(&self, si_to_fp: &SIToFP) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(si_to_fp.get_result());
        match si_to_fp.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match ty.as_ref() {
                    IntegerType { bits: 0..=32 } => match si_to_fp.get_type(&self.module.types).as_ref() {
                        FPType(FPType::Double) => vec![FcvtDW { ret: ret_handler, arg: v_handler }],
                        FPType(FPType::Single) => vec![FcvtSW { ret: ret_handler, arg: v_handler }],
                        _ => todo!()
                    },
                    IntegerType { bits: 33..=64 } => match si_to_fp.get_type(&self.module.types).as_ref() {
                        FPType(FPType::Double) => vec![FcvtDL { ret: ret_handler, arg: v_handler }],
                        FPType(FPType::Single) => vec![FcvtSL { ret: ret_handler, arg: v_handler }],
                        _ => todo!()
                    },
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Int { bits: 0..=64, value } => match si_to_fp.get_type(&self.module.types).as_ref() {
                    FPType(FPType::Double) => vec![MoviI64 { ret: ret_handler, arg: (*value as i64 as f64).to_bits() as i64 }],
                    FPType(FPType::Single) => vec![MoviI64 { ret: ret_handler, arg: (*value as i64 as f32).to_bits() as i64 }],
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    pub fn ui_to_fp(&self, ui_to_fp: &UIToFP) -> Vec<Tcg> {
        let ret_handler = self.name_to_handler(ui_to_fp.get_result());
        match ui_to_fp.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match ty.as_ref() {
                    IntegerType { bits: 0..=32 } => match ui_to_fp.get_type(&self.module.types).as_ref() {
                        FPType(FPType::Double) => vec![FcvtDW { ret: ret_handler, arg: v_handler }],
                        FPType(FPType::Single) => vec![FcvtSW { ret: ret_handler, arg: v_handler }],
                        _ => todo!()
                    },
                    IntegerType { bits: 33..=64 } => match ui_to_fp.get_type(&self.module.types).as_ref() {
                        FPType(FPType::Double) => vec![FcvtDL { ret: ret_handler, arg: v_handler }],
                        FPType(FPType::Single) => vec![FcvtSL { ret: ret_handler, arg: v_handler }],
                        _ => todo!()
                    },
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Int { bits: 0..=64, value } => match ui_to_fp.get_type(&self.module.types).as_ref() {
                    FPType(FPType::Double) => vec![MoviI64 { ret: ret_handler, arg: (*value as f64).to_bits() as i64 }],
                    FPType(FPType::Single) => vec![MoviI64 { ret: ret_handler, arg: (*value as f32).to_bits() as i64 }],
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}
