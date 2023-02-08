# tek5030 in Rust

## install dependencies

I use Conan to install the OpenCV and other system dependencies.
Additionally, to create the Cargo build script which performs linking, I parse the `conanbuildinfo.txt` to tell
the `opencv` crate about link information.
Shortcuts via [`justfile`](https://github.com/casey/just) are available.
I use `alias j=just`.

```shell
j install # uses conan to install dependencies to the build directory
j apply   # parses conanbuildinfo.txt and sets environments variable within the .cargo/config.toml env section
```

_This is work in progress, since I have some trouble linking `libfreetype`._
