use clap::Parser;
use llvm_ir::Module;

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
    /// 指定输出文件名，不指定时将输出到 `1.c`。
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
    /// 指定函数名，不指定时为 `op`。
    #[arg(short, long, default_value_t = String::from_str("op").unwrap())]
    func: String,
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
        None => panic!("没有找到名为 `op` 的函数"),
    };

    todo!();

    Ok(())
}
