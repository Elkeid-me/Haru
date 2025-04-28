mod process;

use clap::Parser;
use llvm_ir::Module;
use std::{fs::File, io::Write, str::FromStr};

/// Haru：QEMU Patch 自动生成器
#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// 指定输入为二进制格式的 LLVM IR，不指定时，将根据输入文件扩展名猜测。
    #[arg(short, long, default_value_t = false)]
    bc: bool,
    /// 指定输入为文本格式的 LLVM IR，不指定时，将根据输入文件扩展名猜测。
    #[arg(short, long, default_value_t = false)]
    ll: bool,
    /// 指定函数名。
    #[arg(short, long, default_value_t = String::from_str("op").unwrap())]
    func: String,
    /// 指定输出指令名，不指定时与输入函数同名
    #[arg(short, long)]
    inst: Option<String>,
    /// 指定输出文件名，不指定时将输出到 `trans_<INST>.c`。
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
    input: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let module = match (args.bc, args.ll) {
        (true, true) => panic!("那我问你，一个 LLVM IR 文件怎么既是二进制又是文本"),
        (true, false) => Module::from_bc_path(args.input)?,
        (false, true) => Module::from_ir_path(args.input)?,
        // 这里假定 `OsStr` 一定是合法的 Unicode
        (false, false) => match args.input.extension().map(|ex| ex.to_str().unwrap()) {
            Some("ll") => Module::from_ir_path(args.input)?,
            Some("bc") => Module::from_bc_path(args.input)?,
            _ => panic!("未知的扩展名，猜不出来喵"),
        },
    };

    let func_name = args.func.as_str();

    let op_function = match module.get_func_by_name(func_name) {
        Some(func) => func,
        None => panic!("没有找到名为 `{func_name}` 的函数",),
    };

    let inst = args.inst.unwrap_or(func_name.to_string());
    let mut f = File::create(args.output.unwrap_or(std::path::PathBuf::from(format!("trans_{inst}.c"))))?;

    let mut processor = process::Processor::new(&module);
    let result = processor.process_func(op_function);

    let ret_handler = processor.ret;

    writeln!(f, "static bool trans_{inst}(DisasContext *ctx, arg_{inst} *a)",)?;
    writeln!(f, "{{")?;
    writeln!(f, "#ifndef TARGET_RISCV32")?;
    let mut arg_cnt = 1;
    for handler in processor.parameters.iter() {
        match processor.symbol_table.borrow().get(handler).unwrap().as_ref() {
            llvm_ir::Type::IntegerType { bits: 0..=32 } => {
                writeln!(f, "#ifdef TARGET_RISCV32")?;
                writeln!(f, "TCGv_i32 val_{handler} = get_gpr(ctx, a->rs{arg_cnt}, EXT_NONE);")?;
                writeln!(f, "#else")?;
                writeln!(f, "TCGv_i64 rs_{arg_cnt} = get_gpr(ctx, a->rs{arg_cnt}, EXT_NONE);")?;
                writeln!(f, "TCGv_i32 val_{handler} = tcg_temp_new_i32();")?;
                writeln!(f, "tcg_gen_extrl_i64_i32(val_{handler}, rs_{arg_cnt});")?;
                writeln!(f, "#endif")?;
                arg_cnt += 1;
            }
            llvm_ir::Type::IntegerType { bits: 0..=64 } => {
                writeln!(f, "#ifdef TARGET_RISCV32")?;
                writeln!(f, "TCGv_i64 val_{handler} = get_gpr_pair(ctx, a->rs{arg_cnt}, EXT_NONE);")?;
                writeln!(f, "#else")?;
                writeln!(f, "TCGv_i64 val_{handler} = get_gpr(ctx, a->rs{arg_cnt}, EXT_NONE);")?;
                writeln!(f, "#endif")?;
                arg_cnt += 1;
            }
            llvm_ir::Type::FPType(llvm_ir::types::FPType::Single) => {
                writeln!(f, "TCGv_i64 val_{handler} = get_fpr_hs(ctx, a->rs{arg_cnt});")?;
                arg_cnt += 1;
            }
            llvm_ir::Type::FPType(llvm_ir::types::FPType::Double) => {
                writeln!(f, "TCGv_i64 val_{handler} = get_fpr_d(ctx, a->rs{arg_cnt});")?;
                arg_cnt += 1;
            }
            _ => todo!(),
        }
    }

    match &op_function.return_type.as_ref() {
        llvm_ir::Type::IntegerType { bits: 0..=32 } => {
            writeln!(f, "#ifdef TARGET_RISCV32")?;
            writeln!(f, "TCGv_i32 val_{ret_handler} = tcg_temp_new_i32();")?;
            writeln!(f, "#else")?;
            writeln!(f, "TCGv_i64 val_{ret_handler} = tcg_temp_new_i64();")?;
            writeln!(f, "#endif")?;
        }
        llvm_ir::Type::IntegerType { bits: 33..=64 }
        | llvm_ir::Type::FPType(llvm_ir::types::FPType::Double)
        | llvm_ir::Type::FPType(llvm_ir::types::FPType::Single) => {
            writeln!(f, "TCGv_i64 val_{ret_handler} = tcg_temp_new_i64();")?
        }
        _ => todo!(),
    }

    for (handler, ty) in processor.symbol_table.borrow().iter() {
        if !processor.parameters.contains(handler) && *handler != ret_handler {
            match ty.as_ref() {
                llvm_ir::Type::IntegerType { bits: 0..=32 } => writeln!(f, "TCGv_i32 val_{handler} = tcg_temp_new_i32();")?,

                llvm_ir::Type::IntegerType { bits: 33..=64 }
                | llvm_ir::Type::FPType(llvm_ir::types::FPType::Double)
                | llvm_ir::Type::FPType(llvm_ir::types::FPType::Single) => {
                    writeln!(f, "TCGv_i64 val_{handler} = tcg_temp_new_i64();")?
                }

                _ => todo!(),
            }
        }
    }
    for i in result {
        writeln!(f, "{i}")?;
    }
    writeln!(f, "#endif")?;
    writeln!(f, "return true;")?;
    writeln!(f, "}}")?;
    Ok(())
}
