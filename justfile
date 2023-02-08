
alias install := conan-install
alias apply := apply-opencv-config

# list available commands
default:
    @just --list --unsorted

# install conan dependencies into build directory
conan-install *args:
    conan install . \
        --install-folder {{justfile_directory()}}/build \
        --output-folder {{justfile_directory()}}/build \
        -bmissing \
        {{args}}

# install conan dependencies into build directory and force building of all dependencies from source
conan-install-force-build:
    @just conan-install --build

# cargo run release and quiet
cargo-run *args:
    @cargo run -qr {{args}}

# create an opencv_envs.toml file in the build directory that contains linking information for the opencv rust crate, based on the conan dependencies
create-opencv-envs: _create-build-directory
    @just cargo-run --bin get_conan_libs -- \
        {{justfile_directory()}}/build/conanbuildinfo.txt \
        -i OPENCV_INCLUDE_PATHS \
            --ai /usr/include \
        -d OPENCV_LINK_PATHS \
            --ad /usr/lib \
            --ad /lib \
        -l OPENCV_LINK_LIBS \
            -s \
        > {{justfile_directory()}}/build/opencv_envs.toml \
        && echo "Wrote env section contents to {{justfile_directory()}}/build/opencv_envs.toml"

# write env contents from opencv_envs.toml to the .cargo/config.toml env section
apply-opencv-config: create-opencv-envs _create-cargo-directory _create-build-directory
    @just cargo-run --bin apply_to_env -- \
        {{justfile_directory()}}/build/opencv_envs.toml \
        {{justfile_directory()}}/.cargo/config.toml

# remove the build directory
clean-build:
    rm -rf {{justfile_directory()}}/build

# create the .cargo directory
_create-cargo-directory:
    @mkdir -p {{justfile_directory()}}/.cargo

# create the build directory
_create-build-directory:
    @mkdir -p {{justfile_directory()}}/build
