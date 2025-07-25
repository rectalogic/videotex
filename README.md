Repro for https://github.com/gfx-rs/wgpu/issues/8005

```sh-session
$ wasm-pack build --target web --out-dir web/pkg
$ python -m http.server -d web
```
