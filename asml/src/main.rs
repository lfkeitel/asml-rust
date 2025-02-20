extern crate clap;

mod compiler;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use clap::{App, AppSettings, Arg, SubCommand};

const ASML_HEADER: &str = "ASML";

fn main() {
    let app = App::new("ASML")
        .version("0.1.0")
        .author("Lee Keitel")
        .arg(Arg::with_name("INPUT").required(true))
        .subcommand(
            SubCommand::with_name("compile")
                .about("Compile an ASML file to srecord format")
                .arg(Arg::with_name("output").short("o").default_value("stdout"))
                .arg(Arg::with_name("INPUT").required(true)),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Compile and execute an ASML file")
                .arg(Arg::with_name("INPUT").required(true)),
        )
        .settings(&[
            AppSettings::ArgsNegateSubcommands,
            AppSettings::SubcommandsNegateReqs,
        ])
        .get_matches();

    if let Some(subcmd) = app.subcommand_matches("compile") {
        compile_file(
            subcmd.value_of("INPUT").unwrap(),
            subcmd.value_of("output").unwrap(),
        );
    } else if let Some(subcmd) = app.subcommand_matches("run") {
        run_file(subcmd.value_of("INPUT").unwrap());
    } else {
        exec_srecord(app.value_of("INPUT").unwrap());
    }
}

fn compile_file(path: &str, output: &str) {
    println!("Compiling {}", path);
    let src_path = Path::new(path);
    let code = compiler::compile_file(src_path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    write_code_to_file(&code, output);
}

fn write_code_to_file(code: &[asml_vm::CodeSection], output: &str) {
    let mut records = srecord::Srecord(Vec::with_capacity(code.len()));
    records.add_header(ASML_HEADER);

    let mut line_count = 0;

    for part in code {
        let mut total_len = part.code.len();
        let mut pc = part.org;
        let mut i = 0;

        while total_len > 252 {
            records.add_record16(srecord::SrecType::SrecData16, pc, &part.code[i..i + 252]);
            pc += 252;
            i += 252;
            total_len -= 252;
            line_count += 1;
        }

        records.add_record16(srecord::SrecType::SrecData16, pc, &part.code[i..]);
        line_count += 1;
    }

    if line_count < 0xFFFF {
        records.add_record16(srecord::SrecType::SrecCount16, line_count as u16, &[]);
    }
    records.add_record16(srecord::SrecType::SrecStart16, 0, &[]);

    if output == "stdout" {
        print!("{}", records);
    } else if let Ok(file) = File::create(Path::new(output)) {
        match write!(&file, "{}", records) {
            Ok(_) => println!("Compile successful"),
            Err(e) => println!("{}", e),
        }
    } else {
        eprintln!("Unable to open file {}", output);
    }
}

fn run_file(path: &str) {
    println!("Compiling {}", path);
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

    for r in records.0 {
        code.push(asml_vm::CodeSection {
            org: r.address as u16,
            code: r.data,
        });
    }

    execute_code(&code);
}

fn execute_code(code: &[asml_vm::CodeSection]) {
    let mut vm = asml_vm::VM::new();
    vm.install_code(code);

    if vm.run().is_ok() {
        println!("{}", vm.output());
    }
}
