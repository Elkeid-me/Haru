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
        (true, true) => {
            eprintln!("那我问你，一个 LLVM IR 文件怎么既是二进制又是文本.");
            std::process::exit(1);
        }
        (true, false) => Module::from_bc_path(args.input)?,
        (false, true) => Module::from_ir_path(args.input)?,
        // 这里假定 `OsStr` 一定是合法的 Unicode
        (false, false) => match args.input.extension().map(|ex| ex.to_str().unwrap()) {
            Some("ll") => Module::from_ir_path(args.input)?,
            Some("bc") => Module::from_bc_path(args.input)?,
            ext => {
                eprintln!("错误：未指定扩展名 `{}` 未知.", ext.unwrap_or(""));
                std::process::exit(1);
            }
        },
    };

    let func_name = args.func.as_str();

    let op_function = match module.get_func_by_name(func_name) {
        Some(func) => func,
        None => {
            eprintln!("没有找到名为 `{func_name}` 的函数");
            std::process::exit(1);
        }
    };

    let inst = args.inst.unwrap_or(func_name.to_string());
    let mut f = File::create(args.output.as_ref().unwrap_or(&std::path::PathBuf::from(format!("trans_{inst}.c"))))?;

    let mut processor = process::Processor::new(&module);
    let result = processor.process_func(op_function);

    let ret_handler = processor.ret;

    writeln!(f, "static bool trans_{inst}(DisasContext *ctx, arg_{inst} *a)",)?;
    writeln!(f, "{{")?;
    writeln!(f, "#ifndef TARGET_RISCV32")?;
    let mut arg_cnt = 1;
    let mut int_arg_code = 10u32;
    let mut fp_arg_code = 10u32;
    let mut args_code = Vec::new();
    for handler in processor.parameters.iter() {
        match processor.symbol_table.borrow().get(handler).unwrap().as_ref() {
            llvm_ir::Type::IntegerType { bits: 0..=32 } => {
                if processor.used.borrow().contains(handler) {
                    writeln!(f, "#ifdef TARGET_RISCV32")?;
                    writeln!(f, "TCGv_i32 rs_{arg_cnt} = get_gpr(ctx, a->rs{arg_cnt}, EXT_NONE);")?;
                    // TODO: 符号扩展/零扩展
                    writeln!(f, "#else")?;
                    writeln!(f, "TCGv_i64 val_{handler} = get_gpr(ctx, a->rs{arg_cnt}, EXT_NONE);")?;
                    writeln!(f, "#endif")?;
                }
                arg_cnt += 1;
                args_code.push(int_arg_code);
                int_arg_code += 1;
            }
            llvm_ir::Type::IntegerType { bits: 33..=64 } => {
                if processor.used.borrow().contains(handler) {
                    writeln!(f, "#ifdef TARGET_RISCV32")?;
                    writeln!(f, "TCGv_i64 val_{handler} = get_gpr_pair(ctx, a->rs{arg_cnt}, EXT_NONE);")?;
                    writeln!(f, "#else")?;
                    writeln!(f, "TCGv_i64 val_{handler} = get_gpr(ctx, a->rs{arg_cnt}, EXT_NONE);")?;
                    writeln!(f, "#endif")?;
                }
                arg_cnt += 1;
                args_code.push(int_arg_code);
                int_arg_code += 1;
            }
            llvm_ir::Type::FPType(llvm_ir::types::FPType::Single) => {
                if processor.used.borrow().contains(handler) {
                    writeln!(f, "TCGv_i64 val_{handler} = get_fpr_hs(ctx, a->rs{arg_cnt});")?;
                }
                arg_cnt += 1;
                args_code.push(fp_arg_code);
                fp_arg_code += 1;
            }
            llvm_ir::Type::FPType(llvm_ir::types::FPType::Double) => {
                if processor.used.borrow().contains(handler) {
                    writeln!(f, "TCGv_i64 val_{handler} = get_fpr_d(ctx, a->rs{arg_cnt});")?;
                }
                arg_cnt += 1;
                args_code.push(fp_arg_code);
                fp_arg_code += 1;
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

    for handler in processor.symbol_table.borrow().keys() {
        if !processor.parameters.contains(handler) && *handler != ret_handler && processor.used.borrow().contains(handler) {
            writeln!(f, "TCGv_i64 val_{handler} = tcg_temp_new_i64();")?;
        }
    }

    if processor.use_float {
        writeln!(f, "gen_set_rm(ctx, a->rm);")?;
    }

    for i in result {
        writeln!(f, "{i}")?;
    }
    writeln!(f, "#endif")?;
    writeln!(f, "return true;")?;
    writeln!(f, "}}")?;

    let mut f2 = File::create(args.output.as_ref().unwrap_or(&std::path::PathBuf::from(format!("{inst}.type"))))?;

    match processor.parameters.len() {
        0 => writeln!(f2, "{}", if processor.use_float { "r0_rm" } else { "r0" })?,
        1 => writeln!(f2, "{}", if processor.use_float { "r1_rm" } else { "r1" })?,
        2 => writeln!(f2, "{}", if processor.use_float { "r2_rm" } else { "r2" })?,
        3 => writeln!(f2, "{}", if processor.use_float { "r3_rm" } else { "r3" })?,
        _ => todo!(),
    }
    Ok(())
}
