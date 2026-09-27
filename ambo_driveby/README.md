# Ambo Driveby

A macroquad animation built for the web. The build is driven by the
[just](https://github.com/casey/just) recipes in the `justfile`.

## Building

```shell
just build-web
```

This compiles `src/main.rs` for `wasm32-unknown-unknown`, shrinks the binary
and writes everything needed to deploy the animation to
`target/wasm32-unknown-unknown/web-release/`:

- `index.html` – the page that hosts the canvas and boots the wasm
- `ambo_driveby.wasm` – the compiled animation
- `mq_js_bundle.js` – macroquad's JS runtime

The JS bundle is a static asset: `build-web` copies it as-is and only the HTML
is generated per crate. To test the build locally run:

```shell
just serve
```

and open <http://localhost:8000>.

## Multiple canvases

`mq_js_bundle.js` exports `load(wasmPath, canvasId)` and every call creates a
fully independent instance (its own WebGL context, wasm memory and event
handlers). One bundle can therefore serve any number of canvases; a page only
needs to pass a different canvas id to each call:

```html
<canvas id="one" tabindex="1"></canvas>
<canvas id="two" tabindex="1"></canvas>
<script type="module">
    import { load } from './mq_js_bundle.js';

    load('./animation.wasm', 'one');
    load('./animation.wasm', 'two');
</script>
```

## Updating the JS bundle

The upstream [macroquad](https://github.com/not-fl3/macroquad) bundle binds to
a single hard-coded canvas and keeps all of its state in module scope. The
copy in `assets/mq_js_bundle/` is therefore wrapped by
`tools/build_js_bundle.py`, which turns it into a module where each `load()`
call gets its own instance. To fetch the latest upstream bundle and rebuild
the wrapper run:

```shell
just update-js-bundle
```

The script fails loudly if upstream changes in a way it does not understand,
so the patches can be reviewed instead of silently applying to something else.

## WebGL2

`window_conf()` requests WebGL2 in the browser. mqanim renders into a
multisampled render target for anti-aliasing and WebGL1 has no multisampled
render targets, so WebGL1 would fall back to FXAA (or plain aliased output).
