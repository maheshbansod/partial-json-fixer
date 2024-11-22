## partial-json-fixer

This project fixes any partial json.

It exports a function `fix_json` which accepts a string and returns a complete JSON.

It's lenient in some cases so **do not rely on this for checking JSON**.

This package is intended to be used to complete partial JSONs coming from streams.
