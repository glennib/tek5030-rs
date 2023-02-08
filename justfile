
alias install := conan-install
alias apply := apply-opencv-config

default:
    @just --list --unsorted

# install conan dependencies into build directory
conan-install *args:
    conan install . --install-folder {{justfile_directory()}}/build --output-folder {{justfile_directory()}}/build -bmissing {{args}}

# install conan dependencies into build directory and force building of all dependencies from source
conan-install-force-build:
    @just conan-install --build

# cargo run release and quiet
cargo-run *args:
    cargo run -qr {{args}}

# create an opencv_envs.toml file in the build directory that contains linking information for the opencv rust crate, based on the conan dependencies
create-opencv-envs:
    just cargo-run --bin get_conan_libs -- \
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

apply-opencv-config: create-opencv-envs create-cargo-directory
    just cargo-run --bin apply_to_env -- {{justfile_directory()}}/build/opencv_envs.toml {{justfile_directory()}}/.cargo/config.toml

clean-build:
    rm -rf {{justfile_directory()}}/build

create-cargo-directory:
    mkdir -p {{justfile_directory()}}/.cargo
