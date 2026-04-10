#![cfg(feature = "cranelift")]

use bf::{
    backend::{CompilerOptions, cranelift::jit_compile_run},
    opt::v1::Optimizer,
    parse,
    testing::BufTestingIo,
};
use dir_test::{Fixture, dir_test};
use std::{fs, path::PathBuf};

macro_rules! test_dir {
    ($name: ident = $dir: expr) => {
        #[dir_test(
                                    dir: $dir,
                                    glob: "*.b",
                                )]
        fn $name(fixture: Fixture<&str>) {
            let opts = CompilerOptions::default();

            let code = PathBuf::from(fixture.path());
            let input = code.with_extension("in");
            let output = code.with_extension("out");

            let input = fs::read(input).ok();
            let output = fs::read(output).ok();

            let program = parse(fixture.content());
            let program = Optimizer::new(&opts, program).run_all().finish();

            let io = BufTestingIo::new();

            if let Some(input) = input {
                io.load_stdin(input);
            }

            jit_compile_run(&program, opts, Some(Box::new(&io)));

            let outbuf = io.finish();

            if let Some(output) = output {
                assert_eq!(outbuf, output, "Outputs did not match!");
            }
        }
    };
}

test_dir!(basic = "$CARGO_MANIFEST_DIR/tests/fixtures/basic");
