# Concavum customizer

## Overview

This repository contains the source code of the interactive customizer for the [Concavum](https://github.com/julianschuler/keyboards/tree/master/concavum), a fully parametric split keyboard featuring an ergonomic layout with ortholinear (non-staggered) columns and a concave key wells.
The customizer allows for changing all kinds of parameters like the number of columns, rows and thumb keys, the curvature, the distance between keys and many more.

You can use the customizer directly in the browser by heading over to [https://julianschuler.github.io/concavum-customizer]. For better performance, you can also run it natively as described in the following section.

## Running the customizer natively

To run the customizer natively, start by installing [rust](https://www.rust-lang.org/tools/install).
Clone the repository, switch to it and run the customizer using the following:

```sh
git clone https://github.com/julianschuler/concavum-customizer
cd concavum-customizer
cargo run --release
```

## Running the customizer in a browser

The customizer can also be run in a browser by compiling it to WebAssembly.
Start by installing [rustup](https://www.rust-lang.org/tools/install) and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).
Clone the repository, switch to it and build the code using `wasm-pack`:

```sh
git clone https://github.com/julianschuler/concavum-customizer
cd concavum-customizer
wasm-pack build customizer --target no-modules --out-dir ../web/pkg --no-typescript --no-pack
```

The files in the `web` subfolder can now be served using any HTTP server, e.g. if you have Python installed:

```sh
python -m http.server -b localhost -d web
```

For the above example, the customizer should now be accessible under [http://localhost:8000].

## License

This project is licensed under the MIT license, see [`LICENSE.txt`](LICENSE.txt) for further information.
