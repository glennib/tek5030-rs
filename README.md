# tek5030 in Rust

## install dependencies

I use Conan to install the OpenCV and other system dependencies.
Additionally, to create the Cargo build script which performs linking, I use the [ConanCargoWrapper](https://github.com/lasote/conan-cargo-wrapper-generator) generator, which must be created manually:

```shell
# cd ~
git clone https://github.com/lasote/conan-cargo-wrapper-generator.git
cd conan-cargo-wrapper-generator
conan create . -bmissing
```

Prior to building anything rusty, invoke the following from the workspace root:

```shell
conan install . -if build
```

Any crate in the workspace must have the following build script in their `Cargo.toml`:
```toml
build = "../build_common.rs"
```
