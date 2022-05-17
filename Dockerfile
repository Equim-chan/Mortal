# syntax=docker/dockerfile:1.4-labs

FROM archlinux:base-devel as libriichi_build

RUN <<EOF
pacman -Syu --noconfirm --needed rust python
pacman -Scc
EOF

WORKDIR /
COPY Cargo.toml Cargo.lock .
COPY libriichi libriichi

RUN cargo build -p libriichi --lib --release

# -----
FROM archlinux:base

RUN <<EOF
pacman -Syu --noconfirm --needed python python-pytorch python-toml python-tqdm tensorboard
pacman -Scc
EOF

WORKDIR /mortal
COPY mortal .
COPY --from=libriichi_build /target/release/libriichi.so .

ENV MORTAL_CFG config.toml
COPY <<'EOF' config.toml
[control]
state_file = '/mnt/mortal.pth'

[resnet]
conv_channels = 192
num_blocks = 40
enable_bn = true
bn_momentum = 0.99
EOF

VOLUME /mnt

ENTRYPOINT ["python", "mortal.py"]
