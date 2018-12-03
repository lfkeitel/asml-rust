use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: asml FILE");
        std::process::exit(1);
    }

    let srec_path = Path::new(&args[1]);

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

    let mut vm = asml_vm::VM::new();
    vm.install_code(&code);

    if let Ok(_) = vm.run() {
        // println!("{:?}", vm);
        println!("{}", vm.output());
    }
}
