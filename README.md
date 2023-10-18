# embedded-rust-os

This is a personal toy project to continue learning Rust and having fun with embedded and operating system

All the ideas, and code comes from this awesome blog:

https://os.phil-opp.com/

I may add some comments to remember things in the code and change slightly some minor details

# issue with Rust-Analyzer (vscode)

Rust analyzer was complaining about "can't find crate for test" with no_std.

Settings seem to change often, see for example:

https://github.com/rust-lang/rust-analyzer/issues/3801

So as for October 2023, I had to add this in my settings.json:

```json
"rust-analyzer.check.targets": "embedded-rust-os"
```