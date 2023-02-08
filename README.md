# tek5030 in Rust

## install dependencies

I use Conan to install the OpenCV and other system dependencies. Additionally, to create the Cargo build script which
performs linking, I parse the `conanbuildinfo.txt` to tell the `opencv` crate about link information. Shortcuts
via [`just`](https://github.com/casey/just#packages) are available. Install instructions are available in that repo, but
the simplest way is to `cargo install just`. I use `alias j=just`.

```shell
j install # uses conan to install dependencies to the build directory
j apply   # parses conanbuildinfo.txt and sets environments variable
          # within the .cargo/config.toml env section
```

### details

Two command line utilities within the [`manage_opencv`](./manage_opencv) crate are used to apply the
Conan-generated OpenCV link targets to Cargo's environment:

* `get_conan_libs` parses `conanbuildinfo.txt` and outputs [TOML](https://toml.io/en/)-compatible lists to standard
  output.
* `apply_to_env` reads such TOML-input and applies them to the `env` section of a project-specific
  Cargo [configuration](https://doc.rust-lang.org/cargo/reference/config.html).

The provided [`justfile`](./justfile) glues this together.

With the `OPENCV_*` environment variables now applied to `.cargo/config.toml`, Cargo will now build
the [`opencv`](https://crates.io/crates/opencv) crate using Conan's OpenCV version.
