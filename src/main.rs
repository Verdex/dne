
mod util;
mod parsing;
mod compiling;
mod eval;

fn main() {


    let ir = parsing::ir_parser::parse("").unwrap();

    let procs = compiling::ir_compiler::compile(&ir).unwrap();

    let vm = eval::vm::Vm::new(procs);

}
