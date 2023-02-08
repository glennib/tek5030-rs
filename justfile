
alias install := conan-install
alias apply := apply-opencv-config

default:
    just --list


conan-install *args:
    conan install . --install-folder {{justfile_directory()}}/build --output-folder {{justfile_directory()}}/build -bmissing {{args}}

conan-install-force-build:
    @just install --build

create-opencv-envs: conan-install
    cargo run -r --bin get_conan_libs {{justfile_directory()}}/build/conanbuildinfo.txt -i OPENCV_INCLUDE_PATHS -d OPENCV_LINK_PATHS -l OPENCV_LINK_LIBS > {{justfile_directory()}}/build/opencv_envs.toml && echo "Wrote env section contents to {{justfile_directory()}}/build/opencv_envs.toml"

apply-opencv-config: create-opencv-envs create-cargo-directory
    cargo run -r --bin apply_to_env {{justfile_directory()}}/build/opencv_envs.toml {{justfile_directory()}}/.cargo/config.toml

clean-build:
    rm -rf {{justfile_directory()}}/build

create-cargo-directory:
    mkdir -p {{justfile_directory()}}/.cargo
