# jamfest

## Running

I used the following guide, with some modifications, to build for the web:

https://bevy-cheatbook.github.io/platforms/wasm.html

After which, you can quickly run the game with:

```
cargo run --target wasm32-unknown-unknown
```

The output of that command will give you a local url that you can open
in a web browser to play the game.

### Troubleshooting

Note if you get an error like:

```shell
Error:

it looks like the Rust project used to create this wasm file was linked against
version of wasm-bindgen that uses a different bindgen format than this binary:

  rust wasm file schema version: 0.2.83
     this binary schema version: 0.2.82

Currently the bindgen format is unstable enough that these two schema versions
must exactly match. You can accomplish this by either updating the wasm-bindgen
dependency or this binary.

You should be able to update the wasm-bindgen dependency with:

    cargo update -p wasm-bindgen

or you can update the binary with

    cargo install -f wasm-bindgen-cli

if this warning fails to go away though and you're not sure what to do feel free
to open an issue at https://github.com/rustwasm/wasm-bindgen/issues!
```

then you need to update your version of `wasm-server-runner`:

```sh
cargo install -f wasm-server-runner
```

## Building

You will need to install the following to perform builds:

```
cargo install -f wasm-bindgen-cli
```

Then to build do:

```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/jamfest.wasm
```

Then create a file called `out/index.html` with the following contents:

```
<!doctype html>
<html lang="en">
<script type="module">
  import init from './jamfest.js'
  init()
</script>

<body style="margin: 0px;">
</body>

</html>
```

Then do:

```
cp -r assets out/
zip -r jamfest.zip out/
```

And finally, `jamfest.zip` can be uploaded to itch.io.

The game currently only supports WASM builds, but we can re-add desktop support pretty easily.
