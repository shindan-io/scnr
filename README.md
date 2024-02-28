# SCNR

When looking for forensics, diagnostics, or any kind of file analysis, it is often necessary to scan through a lot of files, and to be able to query their content. But dealing with a lot of different file formats is such a pain !

This tool aims to simplify the process of scanning files, and querying structured content while getting rid of the file formats complexity.

This process enables then to query (`| grep`, `| jq`) the content of the files with ease.

It aims to transform any file format into one of the 3 following :
- Structured files (even databases) should be converted to json (and thus be queryable with a `jq` filter)
- Text files should stay text
- Any other file or binary should remain binary

## Installation from github (without cloning the repo)

Install Rust : https://www.rust-lang.org/tools/install
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then install `scnr`
```sh
cargo install --git https://github.com/shindan-io/scnr
```


## Installation (from the repo)

`just install`

> Just is a more modern alternative to make : https://github.com/casey/just
> You can install it by `cargo install just`

## Usage

`scnr -h`

```
All in one super awesome file scanner

Usage: scnr [OPTIONS] [COMMAND]

Commands:
  scan     Scan and output results to the console (allowing you to grep)
  extract  Scan and output results to files in an output directory
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Verbose output - only opt-in traces
  -h, --help     Print help
  -V, --version  Print version
```

`scnr scan -h`

```
Scan and output results to the console (allowing you to grep)

Usage: scnr scan [OPTIONS]

Options:
  -i, --input <INPUT>      Input file or directory to start scanning [default: .]
  -f, --filter <FILTER>    Included glob patterns
  -s, --starter <STARTER>  Adds a starter plugin (one that is not associated with any blog pattern, but will be able to start the recursion, like the file-system plugin) [possible values: file-system, json, zip, tar-gz, tar-xz, text, plist, sqlite, bin]
  -c, --cfg <CFG>          Override default settings by allowing named plugins to handle specific files using glob patterns (e.g. --cfg *.json=json --cfg *data*.sql=sqlite --cfg **/do_not_deser.json=bin).
                           Plugins are added in the inverse order of the command line, but the more precise glob patterns in the end.
  -p, --profile <PROFILE>  Plugins configuration profile to start with. Profiles are cfg bundles and can be then overridden by cfg args [default: standard] [possible values: standard, sysdiagnose, nothing]
  -h, --help               Print help
  -V, --version            Print version
```


`scnr extract -h`

```
Scan and output results to files in an output directory

Usage: scnr extract [OPTIONS] --output <OUTPUT>

Options:
  -i, --input <INPUT>      Input file or directory to start scanning [default: .]
  -f, --filter <FILTER>    Included glob patterns
  -s, --starter <STARTER>  Adds a starter plugin (one that is not associated with any blog pattern, but will be able to start the recursion, like the file system-plugin) [possible values: file-system, json, zip, tar-gz, tar-xz, text, plist, sqlite, bin]
  -c, --cfg <CFG>          Override default settings by allowing named plugins to handle specific files using glob patterns (e.g. --cfg *.json=json --cfg *data*.sql=sqlite --cfg **/do_not_deser.json=bin).
                           Plugins are added in the inverse order of the command line, but the more precise glob patterns in the end.
  -p, --profile <PROFILE>  Plugins configuration profile to start with. Profiles are cfg bundles and can be then overridden by cfg args [default: standard] [possible values: standard, sysdiagnose, nothing]
  -o, --output <OUTPUT>    Output directory to extrat all files
      --force              Force extraction even if the output directory is not empty
  -h, --help               Print help
  -V, --version            Print version
```


## Use cases & Examples

See examples in the [[`examples`]] directory

### Grep through sqlite database

`scnr scan -i _samples -f *.db | grep Islands`

```sh
    "country": "Faroe Islands",
    "country": "Virgin Islands, U.S.",
    "country": "Faroe Islands",
    "country": "Virgin Islands, U.S.",
```

### Recursivly extract and convert all eligible files

```sh
rm -rf target/extracted
RUSTLOG=debug scnr -v extract -i  _samples -o target/extracted
```

### Transform plist to json

`scnr scan -i _samples -f *.plist`


### Jq through plist files

scnr embed a built-in jq implementation and is able to execute jq on any file that can be converted to json.

`scnr jq -i _samples -f '*sampled.plist' -q .Label`

```json
"com.example.sampled"
```


## Python bindings

https://www.infoworld.com/article/3664124/how-to-use-rust-with-python-and-python-with-rust.html



## Backlog - Todos

- [ ] publish on crates.io
- [ ] python bindings / usage as python library / publish on `pypi`
- [ ] js-ts bindings / usage as node library / publish on `npm`
- [ ] better documentation / `rust book` / examples / use cases
- [ ] Handle `stdin` and `stdout` as input and output
- [ ] Handle archives passwords / encryptions

-

- [x] `jq` queries integration 
- [x] more file formats
  - [x] `xml` (to json)
  - [x] `yaml` (to json) 
  - [x] `toml` (to json)
- [x] public repository
- [x] configuration profiles (default or sysdiagnose for now)
- [x] plugins configuration (`file name / plugin used`) with glob pattern
- [x] be able to read through any file formats (`zip, tar, sqlite` ...)
- [x] scan only a subset (`filters`, wildcards)
- [x] cli tool
- [x] usage as rust library(`crate`)

### Long term target
- [ ] `WASM`/`no file system` compat => be able to run in the browser (perhaps with just a subset of plugins ?)
- [ ] Cache system / or at least be able to randomly access anyfile in the input
- [ ] Ability for plugins to handle multiple "files" at the same time (will be usefull to read `unified logs`)


## Other tools & Inspirations

- https://github.com/EC-DIGIT-CSIRC/sysdiagnose/

