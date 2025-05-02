use super::Handler;
use std::fmt::{Display, Formatter, Result};

/// 名义上叫 TCG，实际上因为浮点函数，不是 TCG。
pub enum Tcg {
    ShliI32 { ret: Handler, arg_1: Handler, arg_2: i32 },
    SariI32 { ret: Handler, arg_1: Handler, arg_2: i32 },

    AddiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    AddI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    /// 为了代码的简化，这里 `subfi_i64` 与实际 TCG 对应函数的后两个参数是反过来的
    SubfiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    SubiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    SubI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    MuliI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    MulI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    DivI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    DivuI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    RemI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    RemuI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    AndiI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    AndI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    OriI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    OrI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    XoriI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    XorI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    ShliI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    ShlI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    ShriI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    ShrI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    SariI64 { ret: Handler, arg_1: Handler, arg_2: i64 },
    SarI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    MoviI32 { ret: Handler, arg: i32 },
    MoviI64 { ret: Handler, arg: i64 },
    MovI64 { ret: Handler, arg: Handler },
    ExtactI64 { ret: Handler, arg: Handler, pos: u32, len: u32 },
    ExtrlI64I32 { ret: Handler, arg: Handler },

    SminI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SmaxI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    UminI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    SetcondEqI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SetcondNeI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SetcondUgtI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SetcondUgeI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SetcondUltI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SetcondUleI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SetcondSgtI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SetcondSgeI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SetcondSltI64 { ret: Handler, arg_1: Handler, arg_2: Handler },
    SetcondSleI64 { ret: Handler, arg_1: Handler, arg_2: Handler },

    /// 32 位有符号整数转为单精度浮点数
    FcvtSW { ret: Handler, arg: Handler },
    /// 32 位无符号整数转为单精度浮点数
    FcvtSWu { ret: Handler, arg: Handler },
    /// 64 位有符号整数转为单精度浮点数
    FcvtSL { ret: Handler, arg: Handler },
    /// 64 位无符号整数转为单精度浮点数
    FcvtSLu { ret: Handler, arg: Handler },

    /// 32 位有符号整数转为双精度浮点数
    FcvtDW { ret: Handler, arg: Handler },
    /// 32 位无符号整数转为双精度浮点数
    FcvtDWu { ret: Handler, arg: Handler },
    /// 64 位有符号整数转为双精度浮点数
    FcvtDL { ret: Handler, arg: Handler },
    /// 64 位无符号整数转为双精度浮点数
    FcvtDLu { ret: Handler, arg: Handler },

    FcvtDS { ret: Handler, arg: Handler },
    FcvtSD { ret: Handler, arg: Handler },

    FaddS { ret: Handler, arg_1: Handler, arg_2: Handler },
    FaddD { ret: Handler, arg_1: Handler, arg_2: Handler },

    FsubS { ret: Handler, arg_1: Handler, arg_2: Handler },
    FsubD { ret: Handler, arg_1: Handler, arg_2: Handler },

    FmulS { ret: Handler, arg_1: Handler, arg_2: Handler },
    FmulD { ret: Handler, arg_1: Handler, arg_2: Handler },

    FdivS { ret: Handler, arg_1: Handler, arg_2: Handler },
    FdivD { ret: Handler, arg_1: Handler, arg_2: Handler },

    SetDestGpr { expr: Handler },
    SetDestGprPair { expr: Handler },
    SetDestFprHs { expr: Handler },
    SetDestFprD { expr: Handler },
    Ret { float: bool },

    RVArc { rv_32: Option<Box<Tcg>>, rv_64: Option<Box<Tcg>> }
}

impl Display for Tcg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::ShliI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_shli_i32(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::SariI32 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_sari_i32(val_{ret}, val_{arg_1}, {arg_2});"),

            Self::AddiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_addi_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::AddI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_add_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::SubfiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_subfi_i64(val_{ret}, {arg_2}, val_{arg_1});"),
            Self::SubiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_subi_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::SubI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_sub_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::MuliI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_muli_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::MulI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_mul_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::DivI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_div_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::DivuI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_divu_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::RemI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_rem_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::RemuI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_remu_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::AndiI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_andi_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::AndI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_and_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::OriI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_ori_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::OrI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_or_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::XoriI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_xori_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::XorI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_xor_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::ShliI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_shli_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::ShlI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_shl_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::ShriI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_shri_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::ShrI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_shr_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::SariI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_sari_i64(val_{ret}, val_{arg_1}, {arg_2});"),
            Self::SarI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_sar_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::MoviI32 { ret, arg } => write!(f, "tcg_gen_movi_i32(val_{ret}, {arg});"),
            Self::MoviI64 { ret, arg } => write!(f, "tcg_gen_movi_i64(val_{ret}, {arg});"),
            Self::MovI64 { ret, arg } => write!(f, "tcg_gen_mov_i64(val_{ret}, val_{arg});"),
            Self::ExtactI64 { ret, arg, pos, len } => write!(f, "tcg_gen_extract_i64(val_{ret}, val_{arg}, {pos}, {len});"),
            Self::ExtrlI64I32 { ret, arg } => write!(f, "tcg_gen_extrl_i64_i32(val_{ret}, val_{arg});"),

            Self::SminI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_smin_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SmaxI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_smax_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::UminI64 { ret, arg_1, arg_2 } => write!(f, "tcg_gen_umin_i64(val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::SetcondEqI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_EQ, val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SetcondNeI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_NE, val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SetcondUgtI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_GTU, val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SetcondUgeI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_GEU, val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SetcondUltI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_LTU, val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SetcondUleI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_LEU, val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SetcondSgtI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_GT, val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SetcondSgeI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_GE, val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SetcondSltI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_LT, val_{ret}, val_{arg_1}, val_{arg_2});"),
            Self::SetcondSleI64 { ret, arg_1, arg_2 } =>
                write!(f, "tcg_gen_setcond_i64(TCG_COND_LE, val_{ret}, val_{arg_1}, val_{arg_2});"),

            Self::FcvtSW { ret, arg } => write!(f, "gen_helper_fcvt_s_w(val_{ret}, tcg_env, val_{arg});"),
            Self::FcvtSWu { ret, arg } => write!(f, "gen_helper_fcvt_s_wu(val_{ret}, tcg_env, val_{arg});"),
            Self::FcvtSL { ret, arg } => write!(f, "gen_helper_fcvt_s_l(val_{ret}, tcg_env, val_{arg});"),
            Self::FcvtSLu { ret, arg } => write!(f, "gen_helper_fcvt_s_lu(val_{ret}, tcg_env, val_{arg});"),

            Self::FcvtDW { ret, arg } => write!(f, "gen_helper_fcvt_d_w(val_{ret}, tcg_env, val_{arg});"),
            Self::FcvtDWu { ret, arg } => write!(f, "gen_helper_fcvt_d_wu(val_{ret}, tcg_env, val_{arg});"),
            Self::FcvtDL { ret, arg } => write!(f, "gen_helper_fcvt_d_l(val_{ret}, tcg_env, val_{arg});"),
            Self::FcvtDLu { ret, arg } => write!(f, "gen_helper_fcvt_d_lu(val_{ret}, tcg_env, val_{arg});"),

            Self::FcvtDS { ret, arg } => write!(f, "gen_helper_fcvt_d_s(val_{ret}, tcg_env, val_{arg});"),
            Self::FcvtSD { ret, arg } => write!(f, "gen_helper_fcvt_s_d(val_{ret}, tcg_env, val_{arg});"),

            Self::FaddS { ret, arg_1, arg_2 } => write!(f, "gen_helper_fadd_s(val_{ret}, tcg_env, val_{arg_1}, val_{arg_2});"),
            Self::FaddD { ret, arg_1, arg_2 } => write!(f, "gen_helper_fadd_d(val_{ret}, tcg_env, val_{arg_1}, val_{arg_2});"),

            Self::FsubS { ret, arg_1, arg_2 } => write!(f, "gen_helper_fsub_s(val_{ret}, tcg_env, val_{arg_1}, val_{arg_2});"),
            Self::FsubD { ret, arg_1, arg_2 } => write!(f, "gen_helper_fsub_d(val_{ret}, tcg_env, val_{arg_1}, val_{arg_2});"),

            Self::FmulS { ret, arg_1, arg_2 } => write!(f, "gen_helper_fmul_s(val_{ret}, tcg_env, val_{arg_1}, val_{arg_2});"),
            Self::FmulD { ret, arg_1, arg_2 } => write!(f, "gen_helper_fmul_d(val_{ret}, tcg_env, val_{arg_1}, val_{arg_2});"),

            Self::FdivS { ret, arg_1, arg_2 } => write!(f, "gen_helper_fdiv_s(val_{ret}, tcg_env, val_{arg_1}, val_{arg_2});"),
            Self::FdivD { ret, arg_1, arg_2 } => write!(f, "gen_helper_fdiv_d(val_{ret}, tcg_env, val_{arg_1}, val_{arg_2});"),

            Self::SetDestGpr { expr } => write!(f, "gen_set_gpr(ctx, a->rd, val_{expr});"),
            Self::SetDestGprPair { expr } => write!(f, "gen_set_gpr_pair(ctx, a->rd, val_{expr});"),
            Self::SetDestFprHs { expr } => write!(f, "gen_set_fpr_hs(ctx, a->rd, val_{expr});"),
            Self::SetDestFprD { expr } => write!(f, "gen_set_fpr_d(ctx, a->rd, val_{expr});"),
            Self::Ret { float } => if *float {
                write!(f, "mark_fs_dirty(ctx);\nreturn true;")
            } else {
                write!(f, "return true;")
            },

            Self::RVArc { rv_32, rv_64 } => match (rv_32, rv_64) {
                (Some(rv_32), Some(rv_64)) => write!(f, "#ifdef TARGET_RISCV32\n{rv_32}\n#else\n{rv_64}\n#endif"),
                (Some(rv_32), None) => write!(f, "#ifdef TARGET_RISCV32\n{rv_32}\n#endif"),
                (None, Some(rv_64)) => write!(f, "#ifndef TARGET_RISCV32\n{rv_64}\n#endif"),
                (None, None) => unreachable!(),
            },
        }
    }
}

impl Tcg {
    pub fn rv_arc(rv_32: Tcg, rv_64: Tcg) -> Self {
        Self::RVArc { rv_32: Some(Box::new(rv_32)), rv_64: Some(Box::new(rv_64)) }
    }
}
