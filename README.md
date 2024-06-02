# Deduplication of yara rules

This script takes a path of yara rules, and goes over them to identify duplicate rules if any. It then organises the output at a different output file.  
It also organises & creates:
- one single file with all the rules squeezed in
- compiles and saves the compiled yara file

### Requirements
- Rust 1.70+
- Yara-X
- Jansson
  - macOS: `brew install jansson`
  - Linux: `apt -y install libjansson-dev libjansson4`
- Libmagic
  - macOS: `brew install libmagic`
  - Linux: `apt -y install libmagic1 libmagic-dev`


### Running the program
- Help
```bash
./target/release/yara_dedupe -h
Dedup yara rules and compile

Usage: yara_dedupe <COMMAND>

Commands:
  dedupe   Remove duplicates from a vector of YARA rules
  compile  Compile a YARA rule into a binary format
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

- Deduplicating
```bash
./target/release/yara_dedupe dedupe -i data -o all.yara
[* examining: data/email/general_phish.yar                                                                         ]
* Total files processed: 51
* Total yara rules: 5546
* Total yara rules after dedupe: 5535
* Output yara file stored in: all.yara
```

- Compiling the rules
```bash
./target/release/yara_dedupe compile all.yara
* Compiled yara ruleset is stored in: compiled_all.yara
```
---
License: MIT
