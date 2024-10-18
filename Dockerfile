FROM archlinux:latest

# Update system and install necessary packages
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
    base-devel \
    rust \
    git \
    curl \
    wget \
    openssl \
    zlib \
    xz

# Install pyenv
RUN curl https://pyenv.run | bash

# Set up pyenv
ENV PYENV_ROOT /root/.pyenv
ENV PATH $PYENV_ROOT/shims:$PYENV_ROOT/bin:$PATH

# Install Python 3.10 using pyenv
RUN pyenv install 3.10.0
RUN pyenv global 3.10.0

# Install pip for the pyenv Python
RUN pip install --upgrade pip

# Install Maturin
RUN pip install maturin

# Set up Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Add Rust targets
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add x86_64-apple-darwin

WORKDIR /app

CMD ["bash"]
