
# partial-json-fixer

This project contains a zero dependency Rust crate to fix a partial JSON string.

It also contains a python package partial-json-fixer which utilizes this Rust crate.

### Add dependency in a Python project

![PyPI - Version](https://img.shields.io/pypi/v/partial-json-fixer)

[Python package: partial-json-fixer](https://pypi.org/project/partial-json-fixer)

```sh
pip install partial-json-fixer
```

### Add dependency in a Rust project

[crate: partial-json-fixer](https://crates.io/crates/partial-json-fixer)

```sh
cargo add partial-json-fixer
```

## Usage

### Python

```python
from partial_json_fixer import fix_json

assert fix_json("{\"key\": \"value") == "{\"key\": \"value\"}"
```

### Rust

```rust
use partial_json_fixer::fix_json

assert_eq!(fix_json("{\"key\": \"value").unwrap(), "{\"key\": \"value\"}")
```
