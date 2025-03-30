use super::Handler;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum Tcg {
    Label(Handler),
    // TempNewI32,
    AddiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    AddI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    // 为了代码的简化，这里 `subfi_i32` 与实际 TCG 对应函数的后两个参数是反过来的
    SubfiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    SubiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    SubI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    MuliI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    MulI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    DivfiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    DiviI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    DivI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    DivufiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    DivuiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    DivuI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    RemfiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    RemiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    RemI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    RemufiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    RemuiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    RemuI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    AndiI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    AndI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    OriI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    OrI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    XoriI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    XorI32 { ret: Handler, arg_1: Handler, arg_2: Handler },

    AddiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    AddI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    SubfiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    SubiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    SubI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    MuliI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    MulI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    DivfiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    DiviI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    DivI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    DivufiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    DivuiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    DivuI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    RemfiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    RemiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    RemI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    RemufiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    RemuiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    RemuI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    AndiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    AndI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    OriI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    OrI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    XoriI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    XorI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    MoviI32 { ret: Handler, arg: i32 },
    MoviI64 { ret: Handler, arg: i64 },
    MovI32 { ret: Handler, arg: Handler },
    MovI64 { ret: Handler, arg: Handler },
    ExtuI32I64 { ret: Handler, arg: Handler },
    ExtI32I64 { ret: Handler, arg: Handler },
    ExtactI32 { ret: Handler, arg: Handler, pos: u32, len: u32 },
    ExtactI64 { ret: Handler, arg: Handler, pos: u32, len: u32 },
    ExtrlI64I32 { ret: Handler, arg: Handler },

    SetDestGpr { expr: Handler },
    Ret,
}

impl Display for Tcg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Label(handler) => write!(f, "label_{handler}:"),

            Self::AddiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_addi_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::AddI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_add_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::SubfiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_subfi_i32(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::SubiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_subi_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::SubI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_sub_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::MuliI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_muli_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::MulI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_mul_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::DivfiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divfi_i32(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::DiviI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divi_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::DivI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_div_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::DivufiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divufi_i32(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::DivuiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divui_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::DivuI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divu_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::RemfiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remfi_i32(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::RemiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remi_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::RemI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_rem_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::RemufiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remufi_i32(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::RemuiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remui_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::RemuI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remu_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::AndiI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_andi_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::AndI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_and_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::OriI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_ori_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::OrI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_or_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::XoriI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_xori_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::XorI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_xor_i32(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::AddiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_addi_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::AddI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_add_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::SubfiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_subfi_i64(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::SubiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_subi_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::SubI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_sub_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::MuliI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_muli_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::MulI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_mul_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::DivfiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divfi_i64(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::DiviI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divi_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::DivI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_div_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::DivufiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divufi_i64(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::DivuiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divui_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::DivuI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divu_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::RemfiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remfi_i64(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::RemiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remi_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::RemI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_rem_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::RemufiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remufi_i64(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::RemuiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remui_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::RemuI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remu_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::AndiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_andi_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::AndI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_and_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::OriI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_ori_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::OrI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_or_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::XoriI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_xori_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::XorI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_xor_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::MoviI32 { ret, arg } => write!(f, "tcg_gen_movi_i32(val_{ret}, {arg});"),
            Self::MoviI64 { ret, arg } => write!(f, "tcg_gen_movi_i64(val_{ret}, {arg});"),
            Self::MovI32 { ret, arg } => write!(f, "tcg_gen_mov_i32(val_{ret}, val_{arg});"),
            Self::MovI64 { ret, arg } => write!(f, "tcg_gen_mov_i64(val_{ret}, val_{arg});"),
            Self::ExtuI32I64 { ret, arg } => write!(f, "tcg_gen_extu_i32_i64(val_{ret}, val_{arg});"),
            Self::ExtI32I64 { ret, arg } => write!(f, "tcg_gen_ext_i32_i64(val_{ret}, val_{arg});"),
            Self::ExtactI32 { ret, arg, pos, len } => write!(f, "tcg_gen_extract_i32(val_{ret}, val_{arg}, {pos}, {len});"),
            Self::ExtactI64 { ret, arg, pos, len } => write!(f, "tcg_gen_extract_i64(val_{ret}, val_{arg}, {pos}, {len});"),
            Self::ExtrlI64I32 { ret, arg } => write!(f, "tcg_gen_extrl_i64_i32(val_{ret}, val_{arg});"),

            Self::SetDestGpr { expr } => write!(f, "gen_set_gpr(ctx, a->rd, val_{expr});"),

            Self::Ret => write!(f, "return true;"),
        }
    }
}
