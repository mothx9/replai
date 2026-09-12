# C SDK source bundle

`replai-c-sdk-0.1.0.tar.gz` is the versioned source distribution for C ABI 1.
It contains the Rust implementation and binding sources, the public header,
locked dependencies, CMake templates and the staging tools needed to produce a
native installation. It is not a universal prebuilt binary archive.

Building the producer artifacts requires Rust 1.98.1, Cargo, a C/C++ toolchain,
Python 3 and CMake. From the extracted SDK root:

```sh
cargo build --locked --release -p replai-c
python3 tools/generate_abi.py --check
python3 tools/stage_c.py --prefix /absolute/empty/prefix
```

The resulting prefix contains the installed consumer contract:

```text
include/replai.h
lib/libreplai_c.a
lib/libreplai_c.so or libreplai_c.dylib
lib/pkgconfig/replai.pc
lib/cmake/replai/replai-config.cmake
lib/cmake/replai/replai-config-version.cmake
lib/cmake/replai/replai-targets.cmake
share/licenses/replai/LICENSE
```

After staging, a C or C++ consumer needs the installed prefix and its native
toolchain; it does not need Rust or the SDK sources.

## pkg-config consumer

```sh
export PKG_CONFIG_PATH=/absolute/prefix/lib/pkgconfig
cc -std=c11 consumer.c $(pkg-config --cflags --libs replai) -o consumer
cc -std=c11 consumer.c $(pkg-config --cflags --static --libs replai) -o consumer-static
```

For an unambiguously static executable when both library forms are installed,
use the absolute archive returned by the installed prefix together with the
remaining `pkg-config --static` flags. The release qualification tool exercises
that exact link and verifies the absence of a dynamic REPLAI dependency.

## CMake consumer

```cmake
cmake_minimum_required(VERSION 3.20)
project(replai_consumer C)
find_package(replai 0.1 CONFIG REQUIRED)
add_executable(consumer main.c)
target_link_libraries(consumer PRIVATE replai::shared)
```

Replace `replai::shared` with `replai::static` for the static artifact. Configure
with `cmake -S . -B build -DCMAKE_PREFIX_PATH=/absolute/prefix`. Both imported
targets carry the public include directory; the static target also carries its
platform system-link requirements.

The package files derive their prefix from their own location. Moving the whole
installation tree preserves both pkg-config and CMake consumption. Do not move
individual files out of the prefix layout.

ABI 1 remains the complete C contract. Rich completion candidates, revisioned
analysis, validation, structured documents and driven embedding are Rust-native
in this release candidate. See the [C ABI contract](c-api.md) for ownership,
lifecycle, caller-buffer and failure rules.
