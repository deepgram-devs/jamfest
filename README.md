# jamfest

I used the following guide, with some modifications, to build for the web:

https://bevy-cheatbook.github.io/platforms/wasm.html

After which, you can quickly run the game with:

```
cargo run --target wasm32-unknown-unknown
```

The output of that command will give you a local url that you can open
in a web browser to play the game.

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
