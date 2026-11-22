//! `legatum-sim scenario-check FILE...`: the fast check for scenario files, on this engine. Each
//! file must be a Lab scenario made for the current rules version whose state loads (hash),
//! is legal and has an objective that parses (`legatum_wasm::scenario::check_file`). Prints `ok<TAB>FILE` or
//! `FAIL<TAB>FILE<TAB>reason` per file; exits 1 when any fails. Takes under a second a file.
//! Used by the pre-merge gate (`tools/gates/scenario.py`).

use legatum_wasm::scenario::check_file;

pub fn main(args: Vec<String>) -> Result<(), String> {
    let files = &args[1..];
    if files.is_empty() {
        return Err("usage: legatum-sim scenario-check FILE...".into());
    }
    let mut failed = 0;
    for file in files {
        let result = std::fs::read_to_string(file)
            .map_err(|e| format!("cannot read: {e}"))
            .and_then(|text| check_file(&text));
        match result {
            Ok(_) => println!("ok\t{file}"),
            Err(reason) => {
                failed += 1;
                println!("FAIL\t{file}\t{}", reason.replace(['\n', '\t'], " "));
            }
        }
    }
    if failed > 0 {
        return Err(format!("{failed} of {} scenario files failed", files.len()));
    }
    Ok(())
}
