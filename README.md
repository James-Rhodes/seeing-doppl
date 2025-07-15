# Macroquad Template

This is a file mainly for my purposes in creating animations for the web using
macroquad.

To build a macroquad project for the web just run:

```shell
just build-web
```

This will produce a js, html and wasm file in the target dir that contains
everything you need to post an animation on the web. To test you can cd into
the directory that contains the js, html and wasm file and run:

```shell
python3 -m http.server
```
