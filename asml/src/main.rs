extern crate clap;

mod compiler;

use std::path::Path;

use clap::{App, AppSettings, Arg, SubCommand};

fn main() {
    let app = App::new("ASML")
        .version("0.1.0")
        .author("Lee Keitel")
        .arg(Arg::with_name("INPUT").required(true))
        .subcommand(
            SubCommand::with_name("compile")
                .about("Compile ASML file")
                .arg(Arg::with_name("INPUT").required(true)),
        )
        .settings(&[
            AppSettings::ArgsNegateSubcommands,
            AppSettings::SubcommandsNegateReqs,
        ])
        .get_matches();

    if let Some(subcmd) = app.subcommand_matches("compile") {
        compile_file(subcmd.value_of("INPUT").unwrap());
    } else {
        exec_srecord(app.value_of("INPUT").unwrap());
    }
}

fn compile_file(path: &str) {
    let src_path = Path::new(path);
    let code = compiler::compile_file(src_path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    execute_code(&code);
}

fn exec_srecord(path: &str) {
    let srec_path = Path::new(path);

    let records = srecord::parse_file(srec_path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let mut code = Vec::new();

    for r in records {
        code.push(asml_vm::CodeSection {
            org: r.address as u16,
            code: r.data,
        });
    }

    execute_code(&code);
}

fn execute_code(code: &asml_vm::Code) {
    let mut vm = asml_vm::VM::new();
    vm.install_code(&code);

    if vm.run().is_ok() {
        println!("{}", vm.output());
    }
}
