use super::int_op::{extract_const_i64, extract_const_u64, sign_extend_const};
use super::tcg::Tcg::{self, *};
use super::{Handler, Processor};
use llvm_ir::instruction::{FPExt, FPToSI, FPToUI, FPTrunc, SExt, SIToFP, Trunc, UIToFP, UnaryOp, ZExt};
use llvm_ir::{Constant, constant::Float, types::FPType};
use llvm_ir::{Operand::*, Type::*, instruction::HasResult};

impl Processor<'_> {
    fn fp64_to_i64(&self, v_handler: Handler, r_handler: Handler) -> Vec<Tcg> {
        let tmp_1 = self.get_tmp::<1>();
        let tmp_2 = self.get_tmp::<2>();
        let tmp_3 = self.get_tmp::<3>();
        self.use_variable(tmp_1);
        self.use_variable(tmp_2);
        self.use_variable(tmp_3);
        self.use_variable(v_handler);
        self.use_variable(r_handler);
        vec![
            Tcg::ExtactI64 { ret: r_handler, arg: v_handler, pos: 0, len: 52 },
            Tcg::OriI64 { ret: r_handler, arg_1: r_handler, arg_2: 1i64 << 52 }, // r_handler = frac
            Tcg::ExtactI64 { ret: tmp_1, arg: v_handler, pos: 52, len: 11 },     // tmp_1 = exp
            // ---
            Tcg::SubfiI64 { ret: tmp_1, arg_1: tmp_1, arg_2: 1075 }, // tmp_1 = 1075 - exp
            Tcg::SariI64 { ret: tmp_2, arg_1: tmp_1, arg_2: 63 },
            Tcg::XoriI64 { ret: tmp_2, arg_1: tmp_2, arg_2: u64::MAX as i64 },
            Tcg::AndI64 { ret: tmp_2, arg_1: r_handler, arg_2: tmp_2 },
            Tcg::ShrI64 { ret: tmp_2, arg_1: tmp_2, arg_2: tmp_1 }, // tmp_2 = 1075 - exp < 0 ? 0 : frac >> (1075 - exp)
            // ---
            Tcg::SubfiI64 { ret: tmp_1, arg_1: tmp_1, arg_2: 0 }, // tmp_1 = exp - 1076
            Tcg::SariI64 { ret: tmp_3, arg_1: tmp_1, arg_2: 63 },
            Tcg::XoriI64 { ret: tmp_3, arg_1: tmp_3, arg_2: u64::MAX as i64 },
            Tcg::AndI64 { ret: tmp_3, arg_1: r_handler, arg_2: tmp_3 },
            Tcg::ShlI64 { ret: tmp_1, arg_1: tmp_3, arg_2: tmp_1 }, // tmp_1 = frac << (exp - 1075)
            // ---
            Tcg::OrI64 { ret: r_handler, arg_1: tmp_1, arg_2: tmp_2 },
            // ---
            Tcg::SariI64 { ret: tmp_1, arg_1: v_handler, arg_2: 63 }, // sign
            Tcg::XorI64 { ret: r_handler, arg_1: r_handler, arg_2: tmp_1 },
            Tcg::ShriI64 { ret: tmp_2, arg_1: tmp_1, arg_2: 63 },
            Tcg::AddI64 { ret: r_handler, arg_1: r_handler, arg_2: tmp_2 },
            // ---
            // Out of range
            Tcg::ExtactI64 { ret: tmp_3, arg: v_handler, pos: 52, len: 11 },
            Tcg::SubfiI64 { ret: tmp_3, arg_1: tmp_3, arg_2: 1085 },
            Tcg::SariI64 { ret: tmp_3, arg_1: tmp_3, arg_2: 63 }, // tmp_3 is cond
            Tcg::XoriI64 { ret: tmp_2, arg_1: tmp_1, arg_2: (u64::MAX >> 1) as i64 }, // `tmp_2` is `i64::MIN` or MAX`
            Tcg::XorI64 { ret: tmp_2, arg_1: tmp_2, arg_2: r_handler },
            Tcg::AndI64 { ret: tmp_2, arg_1: tmp_2, arg_2: tmp_3 },
            Tcg::XorI64 { ret: r_handler, arg_1: tmp_2, arg_2: r_handler },
            // ---
            Tcg::ExtactI64 { ret: tmp_1, arg: v_handler, pos: 52, len: 11 },
            Tcg::SubfiI64 { ret: tmp_2, arg_1: tmp_1, arg_2: 1022 },
            Tcg::SariI64 { ret: tmp_2, arg_1: tmp_2, arg_2: 63 },
            Tcg::AndI64 { ret: r_handler, arg_1: r_handler, arg_2: tmp_2 },
            // ---
            // NaN:
            Tcg::ExtactI64 { ret: tmp_2, arg: v_handler, pos: 0, len: 52 },
            Tcg::SubiI64 { ret: tmp_1, arg_1: tmp_1, arg_2: 0x7ff },
            Tcg::SariI64 { ret: tmp_1, arg_1: tmp_1, arg_2: 63 },
            Tcg::SubfiI64 { ret: tmp_2, arg_1: tmp_2, arg_2: 0 },
            Tcg::SariI64 { ret: tmp_2, arg_1: tmp_2, arg_2: 63 },
            Tcg::XoriI64 { ret: tmp_2, arg_1: tmp_2, arg_2: u64::MAX as i64 },
            Tcg::OrI64 { ret: tmp_1, arg_1: tmp_1, arg_2: tmp_2 }, // cond
            Tcg::XoriI64 { ret: r_handler, arg_1: r_handler, arg_2: (u64::MAX >> 1) as i64 },
            Tcg::AndI64 { ret: r_handler, arg_1: r_handler, arg_2: tmp_1 },
            Tcg::XoriI64 { ret: r_handler, arg_1: r_handler, arg_2: (u64::MAX >> 1) as i64 },
        ]
    }

    fn fp64_to_u64(&self, v_handler: Handler, r_handler: Handler) -> Vec<Tcg> {
        let tmp_1 = self.get_tmp::<1>();
        let tmp_2 = self.get_tmp::<2>();
        let tmp_3 = self.get_tmp::<3>();
        let tmp_4 = self.get_tmp::<4>();
        self.use_variable(tmp_1);
        self.use_variable(tmp_2);
        self.use_variable(tmp_3);
        self.use_variable(tmp_4);
        self.use_variable(v_handler);
        self.use_variable(r_handler);
        vec![
            Tcg::SariI64 { ret: tmp_4, arg_1: v_handler, arg_2: 63 },
            Tcg::XoriI64 { ret: tmp_4, arg_1: tmp_4, arg_2: u64::MAX as i64 },
            Tcg::AndI64 { ret: tmp_4, arg_1: tmp_4, arg_2: v_handler },
            Tcg::ExtactI64 { ret: r_handler, arg: tmp_4, pos: 0, len: 52 },
            Tcg::OriI64 { ret: r_handler, arg_1: r_handler, arg_2: 1i64 << 52 }, // r_handler = frac
            Tcg::ExtactI64 { ret: tmp_1, arg: tmp_4, pos: 52, len: 11 },         // tmp_1 = exp
            // ---
            Tcg::SubfiI64 { ret: tmp_1, arg_1: tmp_1, arg_2: 1075 },
            Tcg::SariI64 { ret: tmp_2, arg_1: tmp_1, arg_2: 63 },
            Tcg::XoriI64 { ret: tmp_2, arg_1: tmp_2, arg_2: u64::MAX as i64 }, // 1075 - exp < 0 ? 0 : 0xfff
            Tcg::AndI64 { ret: tmp_2, arg_1: r_handler, arg_2: tmp_2 },
            Tcg::ShrI64 { ret: tmp_2, arg_1: tmp_2, arg_2: tmp_1 }, // tmp_2 = 1075 - exp < 0 ? 0 : frac >> (1075 - exp)
            // ---
            Tcg::SubfiI64 { ret: tmp_1, arg_1: tmp_1, arg_2: 0 }, // tmp_1 = exp - 1076
            Tcg::SariI64 { ret: tmp_3, arg_1: tmp_1, arg_2: 63 },
            Tcg::XoriI64 { ret: tmp_3, arg_1: tmp_3, arg_2: u64::MAX as i64 }, // exp - 1076 < 0 ? 0 : 0xfff
            Tcg::AndI64 { ret: tmp_3, arg_1: r_handler, arg_2: tmp_3 },
            Tcg::ShlI64 { ret: tmp_1, arg_1: tmp_3, arg_2: tmp_1 },
            // ---
            Tcg::OrI64 { ret: r_handler, arg_1: tmp_1, arg_2: tmp_2 },
            // ---
            // Out of range
            Tcg::ExtactI64 { ret: tmp_3, arg: tmp_4, pos: 52, len: 11 },
            Tcg::SubfiI64 { ret: tmp_3, arg_1: tmp_3, arg_2: 1086 },
            Tcg::SariI64 { ret: tmp_3, arg_1: tmp_3, arg_2: 63 }, // tmp_3 is cond
            Tcg::XoriI64 { ret: tmp_2, arg_1: r_handler, arg_2: u64::MAX as i64 },
            Tcg::AndI64 { ret: tmp_2, arg_1: tmp_2, arg_2: tmp_3 },
            Tcg::XorI64 { ret: r_handler, arg_1: tmp_2, arg_2: r_handler },
            // ---
            Tcg::ExtactI64 { ret: tmp_1, arg: tmp_4, pos: 52, len: 11 },
            Tcg::SubfiI64 { ret: tmp_2, arg_1: tmp_1, arg_2: 1022 },
            Tcg::SariI64 { ret: tmp_2, arg_1: tmp_2, arg_2: 63 },
            Tcg::AndI64 { ret: r_handler, arg_1: r_handler, arg_2: tmp_2 },
        ]
    }

    pub fn fp_to_si(&self, fp_to_si: &FPToSI) -> Vec<Tcg> {
        let r_handler = self.name_to_handler(fp_to_si.get_result());
        let r_ty = self.symbol_table.borrow().get(&r_handler).unwrap().clone();
        self.use_variable(r_handler);
        match fp_to_si.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), r_ty.as_ref()) {
                    (FPType(FPType::Double), IntegerType { bits }) if matches!(bits, 0..=32) => {
                        let mut ret = self.fp64_to_i64(v_handler, r_handler);
                        let tmp_1 = self.get_tmp::<1>();
                        ret.extend([
                            MoviI64 { ret: tmp_1, arg: sign_extend_const(i32::MAX as u64, 32) },
                            Tcg::SminI64 { ret: r_handler, arg_1: tmp_1, arg_2: r_handler },
                            MoviI64 { ret: tmp_1, arg: sign_extend_const(i32::MIN as u64, 32) },
                            Tcg::SmaxI64 { ret: r_handler, arg_1: tmp_1, arg_2: r_handler },
                            ShliI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                            SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        ]);
                        ret
                    }
                    (FPType(FPType::Double), IntegerType { bits }) if matches!(bits, 33..64) => {
                        let mut ret = self.fp64_to_i64(v_handler, r_handler);
                        ret.extend([
                            ShliI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                            SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        ]);
                        ret
                    }
                    (FPType(FPType::Double), IntegerType { bits: 64 }) => self.fp64_to_i64(v_handler, r_handler),
                    (FPType(FPType::Single), IntegerType { bits }) if matches!(bits, 0..=32) => {
                        let mut ret = vec![FcvtDS { ret: v_handler, arg: v_handler }];
                        let tmp_1 = self.get_tmp::<1>();
                        ret.extend(self.fp64_to_i64(v_handler, r_handler));
                        ret.extend([
                            FcvtSD { ret: v_handler, arg: v_handler },
                            MoviI64 { ret: tmp_1, arg: sign_extend_const(i32::MAX as u64, 32) },
                            Tcg::SminI64 { ret: r_handler, arg_1: tmp_1, arg_2: r_handler },
                            MoviI64 { ret: tmp_1, arg: sign_extend_const(i32::MIN as u64, 32) },
                            Tcg::SmaxI64 { ret: r_handler, arg_1: tmp_1, arg_2: r_handler },
                            ShliI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                            SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        ]);
                        ret
                    }
                    (FPType(FPType::Single), IntegerType { bits }) if matches!(bits, 33..64) => {
                        let mut ret = vec![FcvtDS { ret: v_handler, arg: v_handler }];
                        ret.extend(self.fp64_to_i64(v_handler, r_handler));
                        ret.extend([
                            FcvtSD { ret: v_handler, arg: v_handler },
                            ShliI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                            SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        ]);
                        ret
                    }
                    (FPType(FPType::Single), IntegerType { bits: 64 }) => {
                        let mut ret = vec![FcvtDS { ret: v_handler, arg: v_handler }];
                        ret.extend(self.fp64_to_i64(v_handler, r_handler));
                        ret.push(FcvtSD { ret: v_handler, arg: v_handler });
                        ret
                    }
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Float(Float::Double(double)) => match r_ty.as_ref() {
                    IntegerType { bits } => vec![MoviI64 {
                        ret: r_handler,
                        arg: sign_extend_const(extract_const_i64(*double as i64, *bits) as u64, *bits),
                    }],
                    _ => todo!(),
                },
                Constant::Float(Float::Single(single)) => match r_ty.as_ref() {
                    IntegerType { bits } => vec![MoviI64 {
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
        self.use_variable(r_handler);
        match fp_to_ui.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                match (ty.as_ref(), r_ty.as_ref()) {
                    (FPType(FPType::Double), IntegerType { bits }) if matches!(bits, 0..=32) => {
                        let mut ret = self.fp64_to_u64(v_handler, r_handler);
                        let tmp_1 = self.get_tmp::<1>();
                        ret.extend([
                            MoviI64 { ret: tmp_1, arg: u32::MAX as i64 },
                            UminI64 { ret: r_handler, arg_1: r_handler, arg_2: tmp_1 },
                            ExtactI64 { ret: r_handler, arg: r_handler, pos: 0, len: *bits },
                        ]);
                        ret
                    }
                    (FPType(FPType::Double), IntegerType { bits }) if matches!(bits, 33..64) => {
                        let mut ret = self.fp64_to_u64(v_handler, r_handler);
                        ret.push(ExtactI64 { ret: r_handler, arg: r_handler, pos: 0, len: *bits });
                        ret
                    }
                    (FPType(FPType::Double), IntegerType { bits: 64 }) => self.fp64_to_u64(v_handler, r_handler),
                    (FPType(FPType::Single), IntegerType { bits }) if matches!(bits, 0..=32) => {
                        let mut ret = vec![FcvtDS { ret: v_handler, arg: v_handler }];
                        ret.extend(self.fp64_to_u64(v_handler, r_handler));
                        let tmp_1 = self.get_tmp::<1>();
                        ret.extend([
                            FcvtSD { ret: v_handler, arg: v_handler },
                            MoviI64 { ret: tmp_1, arg: u32::MAX as i64 },
                            UminI64 { ret: r_handler, arg_1: r_handler, arg_2: tmp_1 },
                            ExtactI64 { ret: r_handler, arg: r_handler, pos: 0, len: *bits },
                        ]);
                        ret
                    }
                    (FPType(FPType::Single), IntegerType { bits }) if matches!(bits, 33..64) => {
                        let mut ret = vec![FcvtDS { ret: v_handler, arg: v_handler }];
                        ret.extend(self.fp64_to_u64(v_handler, r_handler));
                        ret.extend([
                            FcvtSD { ret: v_handler, arg: v_handler },
                            ExtactI64 { ret: r_handler, arg: r_handler, pos: 0, len: *bits },
                        ]);
                        ret
                    }
                    (FPType(FPType::Single), IntegerType { bits: 64 }) => {
                        let mut ret = vec![FcvtDS { ret: v_handler, arg: v_handler }];
                        ret.extend(self.fp64_to_u64(v_handler, r_handler));
                        ret.push(FcvtSD { ret: v_handler, arg: v_handler });
                        ret
                    }
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Float(Float::Double(double)) => match r_ty.as_ref() {
                    IntegerType { bits } => vec![MoviI64 { ret: r_handler, arg: extract_const_i64(*double as i64, *bits) }],
                    _ => todo!(),
                },
                Constant::Float(Float::Single(single)) => match r_ty.as_ref() {
                    IntegerType { bits } => vec![MoviI64 { ret: r_handler, arg: extract_const_i64(*single as i64, *bits) }],
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
        self.use_variable(r_handler);
        match si_to_fp.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                self.use_variable(v_handler);
                match (ty.as_ref(), r_ty.as_ref()) {
                    (IntegerType { bits }, FPType(FPType::Double)) if matches!(bits, 0..32) => vec![
                        ShliI64 { ret: r_handler, arg_1: v_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        FcvtDW { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits }, FPType(FPType::Single)) if matches!(bits, 0..32) => vec![
                        ShliI64 { ret: r_handler, arg_1: v_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: r_handler, arg_1: r_handler, arg_2: (64 - bits) as i64 },
                        FcvtSW { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits: 32 }, FPType(FPType::Double)) => vec![FcvtDW { ret: r_handler, arg: v_handler }],
                    (IntegerType { bits: 32 }, FPType(FPType::Single)) => vec![FcvtSW { ret: r_handler, arg: v_handler }],
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
        self.use_variable(r_handler);
        match ui_to_fp.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                self.use_variable(v_handler);
                match (ty.as_ref(), r_ty) {
                    (IntegerType { bits }, FPType(FPType::Double)) if matches!(bits, 0..=32) => vec![
                        ExtactI64 { ret: r_handler, arg: v_handler, pos: 0, len: *bits },
                        FcvtDWu { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits }, FPType(FPType::Single)) if matches!(bits, 0..=32) => vec![
                        ExtactI64 { ret: r_handler, arg: v_handler, pos: 0, len: *bits },
                        FcvtSWu { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits }, FPType(FPType::Double)) if matches!(bits, 33..64) => vec![
                        ExtactI64 { ret: r_handler, arg: v_handler, pos: 0, len: *bits },
                        FcvtDLu { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits }, FPType(FPType::Single)) if matches!(bits, 33..64) => vec![
                        ExtactI64 { ret: r_handler, arg: v_handler, pos: 0, len: *bits },
                        FcvtSLu { ret: r_handler, arg: r_handler },
                    ],
                    (IntegerType { bits: 64 }, FPType(FPType::Double)) => vec![FcvtDLu { ret: r_handler, arg: v_handler }],
                    (IntegerType { bits: 64 }, FPType(FPType::Single)) => vec![FcvtSLu { ret: r_handler, arg: v_handler }],
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
        self.use_variable(ret_handler);
        match trunc.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                self.use_variable(v_handler);
                match ty.as_ref() {
                    IntegerType { bits: 0..=64 } => {
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
        self.use_variable(ret_handler);
        match zext.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                self.use_variable(v_handler);
                match ty.as_ref() {
                    IntegerType { bits } => vec![ExtactI64 { ret: ret_handler, arg: v_handler, pos: 0, len: *bits }],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Int { bits, value } => vec![MoviI64 { ret: ret_handler, arg: extract_const_u64(*value, *bits) }],
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
        self.use_variable(ret_handler);
        match sext.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                self.use_variable(v_handler);
                match ty.as_ref() {
                    IntegerType { bits } => vec![
                        ShliI64 { ret: ret_handler, arg_1: v_handler, arg_2: (64 - bits) as i64 },
                        SariI64 { ret: ret_handler, arg_1: ret_handler, arg_2: (64 - bits) as i64 },
                        ExtactI64 { ret: ret_handler, arg: ret_handler, pos: 0, len: ret_bits },
                    ],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match constant.as_ref() {
                Constant::Int { bits, value } => {
                    vec![MoviI64 { ret: ret_handler, arg: extract_const_i64(sign_extend_const(*value, *bits), ret_bits) }]
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
        self.use_variable(ret_handler);
        match fp_ext.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                self.use_variable(v_handler);
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
        self.use_variable(ret_handler);
        match fp_trunc.get_operand() {
            LocalOperand { name, ty } => {
                let v_handler = self.name_to_handler(name);
                self.use_variable(v_handler);
                match (ty.as_ref(), r_ty) {
                    (FPType(FPType::Double), FPType(FPType::Single)) => vec![FcvtSD { ret: ret_handler, arg: v_handler }],
                    _ => todo!(),
                }
            }
            ConstantOperand(constant) => match (constant.as_ref(), r_ty) {
                (Constant::Float(Float::Double(double)), FPType(FPType::Single)) => {
                    vec![MoviI64 { ret: ret_handler, arg: ((*double as f32).to_bits() as u64 | 0xffff_ffff_0000_0000) as i64 }]
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}
