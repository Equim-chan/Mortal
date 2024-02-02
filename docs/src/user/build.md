# Build
## Build required components
### Prerequisites
To build and use Mortal, you need to have a Python environment and an up-to-date Rust compiler. If you plan to train, make sure you have a GPU installed.

It is recommended to use [miniconda](https://docs.conda.io/en/latest/miniconda.html) and [rustup](https://rustup.rs/) to setup the environment.

Instructions below will assume you already have miniconda and Rust installed.

### Clone
```shell
$ git clone https://github.com/Equim-chan/Mortal.git
$ cd Mortal
```

From now on, the root directory of Mortal will be demonstrated as `$MORTAL_ROOT`.

### Create and activate a conda env
> Working directory: `$MORTAL_ROOT`
```shell
$ conda env create -f environment.yml
$ conda activate mortal
```

### Install pytorch
pytorch is not listed as a dependency in `environment.yml` on purpose so that users can install it with their favored ways as per their requirement, hardware and OS.

Check [pytorch's doc](https://pytorch.org/get-started/locally/) on how to install pytorch in your environment. Personally, I recommend installing pytorch with pip.

```admonish tip
Only `torch` is needed. You can skip the installation of `torchvision` and `torchaudio`.
```

### Build and install libriichi
> Working directory: `$MORTAL_ROOT`
```shell
$ cargo build -p libriichi --lib --release
```

For Linux
```shell
$ cp target/release/libriichi.so mortal/libriichi.so
```

For Windows (MSYS2)
```shell
$ cp target/release/riichi.dll mortal/libriichi.pyd
```

### Test the environment
> Working directory: `$MORTAL_ROOT/mortal`
```shell
$ python
Python 3.9.7 | packaged by conda-forge | (default, Sep 29 2021, 19:23:11)
[GCC 9.4.0] on linux
Type "help", "copyright", "credits" or "license" for more information.
>>> import libriichi
>>> help(libriichi)
```

## Optional targets
### Run tests
> Working directory: `$MORTAL_ROOT`
```shell
$ cargo test --workspace --no-default-features --features flate2/zlib -- --nocapture
```

### Run benchmarks
> Working directory: `$MORTAL_ROOT`
```shell
$ cargo test -p libriichi --no-default-features --bench bench
```

### Build executable utilities
> Working directory: `$MORTAL_ROOT`
```shell
$ cargo build -p libriichi --bins --no-default-features --release
$ cargo build -p exe-wrapper --release
```

### Build documentation
> Working directory: `$MORTAL_ROOT/docs`
```shell
$ cargo install mdbook mdbook-admonish mdbook-pagetoc
$ mdbook build
```
