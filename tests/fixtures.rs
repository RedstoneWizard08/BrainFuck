use std::{fs, io::Cursor, path::PathBuf};

use bf::{interp::interpret, optimizer::Optimizer, parse};
use dir_test::{Fixture, dir_test};

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/fixtures",
    glob: "*.b",
)]
fn should_work_correctly(fixture: Fixture<&str>) {
    let code = PathBuf::from(fixture.path());
    let input = code.with_extension("in");
    let output = code.with_extension("out");

    let input = fs::read(input).ok();
    let output = fs::read(output).ok();

    let program = parse(fixture.content());
    let program = Optimizer::new(program).run_all(1, false).finish();
    let mut outbuf = Vec::new();
    let mut inbuf = Cursor::new(input.unwrap_or_default());

    interpret(&program, &mut outbuf, &mut inbuf);

    if let Some(output) = output {
        assert_eq!(outbuf, output, "Outputs did not match!");
    }
}
