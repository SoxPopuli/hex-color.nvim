[private]
default:
    @just -l

[working-directory: 'lib']
build:
    cargo build

[working-directory: 'lib']
build-release:
    cargo build --release

[working-directory: 'lib']
test-release:
    cargo nextest run --release

[working-directory: 'lib']
test:
    cargo nextest run

[working-directory: 'lib']
clean:
    cargo clean

rust_output_dir := 'lib/target'
debug_dir := rust_output_dir / 'debug'
release_dir := rust_output_dir / 'release'
output := 'hex_color_rs.so'

[linux]
_deploy dir:
    cp {{dir / 'libhex_color_rs.so'}} {{output}}

[macos]
_deploy dir:
    cp {{dir / 'libhex_color_rs.dylib'}} {{output}}

[windows]
_deploy dir:
    copy {{dir / 'hex_color_rs.dll'}} {{output}}

deploy-debug: build (_deploy debug_dir)

deploy: build-release (_deploy release_dir) 
