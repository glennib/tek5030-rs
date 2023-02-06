install:
    conan install . -if {{justfile_directory()}}/build

env:
    cargo run --bin opencv_env -- print export > {{justfile_directory()}}/build/opencv_envs

cargo *args: env
    . {{justfile_directory()}}/build/opencv_envs && cargo {{args}}

