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
  -c, --cfg <CFG>          Override default settings by allowing named plugins to handle certain files using glob patterns (e.g. --cfg *.json=json --cfg *data*.sql=sqlite --cfg **/do_not_deser.json=bin)
  -h, --help               Print help
```


## Use cases

### Grep through sqlite database

`scnr -i _samples -f *.db scan | grep Mike`

