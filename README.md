
# partial-json-fixer

![Crates.io MSRV](https://img.shields.io/crates/msrv/partial-json-fixer)
![PyPI - Version](https://img.shields.io/pypi/v/partial-json-fixer)

This project contains a zero dependency Rust crate to fix a partial JSON string.

It also contains a python package partial-json-fixer which utilizes this Rust crate.

### Add dependency in a Python project

```
pip install partial-json-fixer
```

### Add dependency in a Rust project

```
cargo add partial-json-fixer
```

## Usage

### Python

```
from partial_json_fixer import fix_json

assert fix_json("{\"key\": \"value") == "{\"key\": \"value\"}"
```

### Rust

```
use partial_json_fixer::fix_json

assert_eq!(fix_json("{\"key\": \"value"), "{\"key\": \"value\"}")
```
