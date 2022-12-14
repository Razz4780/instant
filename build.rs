use lalrpop::Configuration;

fn main() {
    Configuration::new()
        .use_cargo_dir_conventions()
        .emit_rerun_directives(true)
        .process()
        .unwrap();
}
