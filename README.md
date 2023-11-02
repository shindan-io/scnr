# SCNR

This tool aims to simplify the process of scanning files.

This process enables then to `grep` the content of the files with ease.

It aims to transform each any file format into one of the 3 following :
- Text files should stay text
- Structured files (or databases) should be converted to json
- Any other file or binary should remain binary

## Installation

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
  -i, --input <INPUT>      Input file or directory to start scanning [default: .]
  -f, --filter <FILTER>    Included glob patterns
  -p, --profile <PROFILE>  Plugins configuration profile to start, can be then overriden with cfg args [default: standard] [possible values: standard, sysdiagnose]
  -c, --cfg <CFG>          Override default settings by allowing named plugins to handle specific files using glob patterns (e.g. --cfg *.json=json --cfg *data*.sql=sqlite --cfg **/do_not_deser.json=bin)
  -h, --help               Print help
```


## Use cases

### Grep through sqlite database

`scnr -i _samples -f *.db scan | grep mike`

```sh
    "email": "Mike.Hillyer@sakilastaff.com",
    "first_name": "Mike",
    "username": "Mike"
```


### Recursivly extract and convert all eligible files

`scnr -v -i  _samples extract -f -o target/extracted`


### Jq through plist files

scnr embed a built-in jq implementation and is able to execute jq on any file that can be converted to json.

`scnr -i _samples -f *.plist scan | jq '.SSID'`

```sh

TODO

```



## Backlog - Todos

- [ ] more file formats
  - [ ] `xml` (to json)
  - [ ] `yaml` (to json) 
  - [ ] `toml` (to json)
- [ ] `jq` queries integration 
- [ ] public repository
- [ ] publish on crates.io
- [ ] python bindings / usage as python library / publish on `pypi`
- [ ] js-ts bindings / usage as node library / publish on `npm`
- [ ] better documentation / `rust book` / examples / use cases

- [x] configuration profiles (default or sysdiagnose for now)
- [x] plugins configuration (`file name / plugin used`) with glob pattern
- [x] be able to read through any file formats (`zip, tar, sqlite` ...)
- [x] scan only a subset (`filters`, wildcards)
- [x] cli tool
- [x] usage as rust library(`crate`)

## Long term target
- [ ] `WASM`/`no file system` compat => be able to run in the browser
- [ ] Cache system / or at least be able to randomly access anyfile in the input 
- [ ] Ability for plugins to handle multiple "files" at the same time (will be usefull to read `unified logs`)
