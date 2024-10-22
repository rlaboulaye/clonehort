# ClOneHORT

Deep learning library to create synthetic genomes from real cohort, providing anonymized individual level genomic data.

## Requirements

- OS: Linux and Mac supported
- Compiler: System must have C libraries installed for rust-htslib crate
- Rust: Cargo must be installed to compile and run this code
- Crates: Crate dependencies are listed in Cargo.toml and will automatically be installed prior to compilation

## Compilation

```
cargo build --release --bin
```

## Usage

Choose from the following command options:
- Prepare
- Train
- Generate
- Evaluate
- Compare

Descriptions are available with the --help flag.
```
./target/release/clonehort prepare --help
```

## Example
```
./target/release/clonehort prepare -d ./data/example
./target/release/clonehort train -d ./data/example
./target/release/clonehort generate -d ./data/example
./target/release/clonehort evaluate -d ./data/example
./target/release/clonehort compare -d ./data/example
```
