# Building from Source
To build CSCSCA from source you must:

1) Install Rust (instructions can be found [here](https://rust-lang.org/tools/install/))
2) Open your terminal
    1) Ensure that you have the latest Rust version by running `rustup check` and `rustup update`
    2) Clone this repository by running `git clone https://github.com/cfeyen/cscsca.git`
    3) Enter the repository by running `cd cscsca`
    4) Build CSCSCA by running `cargo build --release`
    5) The executable will located at `.\target\release\cscsca.exe` where `.` is the path to your current working directory
        1) It is recomened to copy this executable into your `PATH` so you can run it by typing `cscsca.exe`