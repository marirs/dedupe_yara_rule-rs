# Deduplication of yara rules

This script takes a path of yara rules, and goes over them to identify duplicate rules if any. It then organises the output at a different output file.  
It also organises & creates:
- one single file with all the rules squeezed in
- compiles and saves the compiled yara file

### Requirements
- Rust 1.50+
- Yara 4.1.1
- Jansson
  - macOS: `brew install jansson`
  - Linux: `apt -y install libjansson-dev libjansson4`
- Libmagic
  - macOS: `brew install libmagic`
  - Linux: `apt -y install libmagic1 libmagic-dev`

### Compiling
- macOS
```bash
YARA_ENABLE_CRYPTO=0 \
YARA_ENABLE_HASH=1 \
YARA_ENABLE_PROFILING=1 \
YARA_ENABLE_MAGIC=1 \
YARA_ENABLE_CUCKOO=1 \
YARA_ENABLE_DOTNET=1 \
YARA_ENABLE_DEX=1 \
YARA_ENABLE_MACHO=1  \
cargo b --release
```

- Linux
```bash
YARA_ENABLE_CRYPTO=1 \
YARA_ENABLE_HASH=1 \
YARA_ENABLE_PROFILING=1 \
YARA_ENABLE_MAGIC=1 \
YARA_ENABLE_CUCKOO=1 \
YARA_ENABLE_DOTNET=1 \
YARA_ENABLE_DEX=1 \
YARA_ENABLE_MACHO=1  \
cargo b --release
```

### Running the program
- Help
```bash
./target/release/yara_dedup -h
yara_dedup 0.1.1

Marirs <marirs@gmail.com>

USAGE:
    yara_dedup [FLAGS] --input-dir <INPUT_DIR> --output-file <OUTPUT_FILE>

FLAGS:
    -c, --compile    enabling this will give a single deduplicated compiled yara file
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
    -i, --input-dir <INPUT_DIR>        directory containing the yara rule files for dedupe
    -o, --output-file <OUTPUT_FILE>    output file name to store the deduplicated single yara file
```

- Deduplicating
```bash
./target/release/yara_dedup -i data -o all.yara
[* examining: data/email/general_phish.yar                                                                         ]
* Total files processed: 51
* Total yara rules: 5546
* Total yara rules after dedupe: 5535
* Output yara file stored in: all.yara
```
---
License: MIT