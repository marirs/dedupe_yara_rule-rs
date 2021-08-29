# YARA-DEDUP


### Requirements
- Rust 1.50+
- Yara 4.1.1

### Compiling
- macOS (`brew install jansson libmagic`)
```bash
YARA_ENABLE_CRYPTO=0 \
YARA_ENABLE_HASH=1 \
YARA_ENABLE_PROFILING=1 \
YARA_ENABLE_MAGIC=1 \
YARA_ENABLE_CUCKOO=1 \
YARA_ENABLE_DOTNET=1 \
YARA_ENABLE_DEX=1 \
YARA_ENABLE_MACHO=1  \
cargo b
```

- Linux (`apt install libjansson-dev libjansson4 libmagic1 libmagic-dev`)
```bash
YARA_ENABLE_CRYPTO=1 \
YARA_ENABLE_HASH=1 \
YARA_ENABLE_PROFILING=1 \
YARA_ENABLE_MAGIC=1 \
YARA_ENABLE_CUCKOO=1 \
YARA_ENABLE_DOTNET=1 \
YARA_ENABLE_DEX=1 \
YARA_ENABLE_MACHO=1  \
cargo b 
```
