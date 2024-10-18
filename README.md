To build for host OS, grab python 3.12 or make a venv
pip install maturin


maturin build --release --target x86_64-apple-darwin -i3.12
maturin build --release --target aarch64-apple-darwin -i3.12

These commands should create a whl file for osx (macos), for intel based macs use the x86_64 whl created

maturin build --release --target x86_64-pc-windows-msvc -i python3.12
This should create the whl file for windows.

To install these, just activate your python env and cd to target/wheels and pip install the whl file that corresponds to your os.
