## partial-json-fixer

Github: https://github.com/maheshbansod/partial-json-fixer

This project fixes any partial json.

It's lenient in some cases so **do not rely on this for checking JSON**.

This package is intended to be used to complete partial JSONs coming from
streams.


### Usage

```python
from partial_json_fixer import fix_json_string

assert fix_json_string("{\"key\": \"value") == "{\"key\": \"value\"}"
```

