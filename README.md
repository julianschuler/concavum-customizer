# Concavum customizer

![An image of the UI of the customizer](img/customizer.png)

## Overview

This repository contains the source code of the interactive customizer for the [Concavum](https://github.com/julianschuler/keyboards/tree/master/concavum), a fully parametric split keyboard featuring an ergonomic layout with ortholinear (non-staggered) columns and concave key wells.
The customizer allows for changing all kinds of parameters like the number of columns, rows and thumb keys, the curvature, the distance between keys and many more.
It generates 3D printing files for the case and a PCB for connecting the key switches.

To get started, head over to the [latest release](https://github.com/julianschuler/concavum-customizer/releases/latest) and download the executable for your operating system. If your operating system is not supported or you want to run the latest development version instead, follow the instructions in the next section.

> **Note:** For Linux, the file dialogs require a compatible XDG Desktop Portal backend to function correctly. If the load/save/export buttons do not seem to work, you might need to install a [different backend](https://docs.rs/rfd/latest/rfd/#xdg-desktop-portal-backend).

## Building the customizer from source

Start by cloning the repository and [installing rust](https://www.rust-lang.org/tools/install).
Run the customizer using the following:

```sh
cargo run --release
```

## Running the customizer in a browser (experimental)

There is experimental support for running the customizer in the browser.
You can find the latest version of the customizer deployed at https://julianschuler.github.io/concavum-customizer.

> **Note:** Due to the current lack of multithreading support in WebAssembly, reloading the model can be easily more than 10x slower when running in the browser compared to running natively.

To run the web version locally, start by building the code using [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/):

```sh
wasm-pack build customizer_wasm --target no-modules --out-dir ../web/pkg --no-typescript --no-pack
```

The files in the `web` subfolder can now be served using any HTTP server, e.g. if you have Python installed:

```sh
python -m http.server -b localhost -d web
```

For the above example, the customizer should now be accessible under http://localhost:8000.

## Using the customizer

The customizer consists of two parts: The configuration panel at the left and the model viewer at the right.
The model is automatically reloaded for any change in the configuration panel.

If you are finished with configuring the keyboard, you can use the export button at the top right of the configuration panel to export all the model files.
The model files are exported as a ZIP archive with the following files:

```
concavum.zip
├── config.toml
├── left_half.stl
├── right_half.stl
├── bottom_plate.stl
├── bottom_plate.stl
├── matrix_pcb.kicad_pcb
└── kikit_parameters.json
```

The `config.toml` file contains all the parameters of the exported model and can be loaded back into the customizer using the load button.
The `left_half.stl`, `right_half.stl` and `bottom_plate.stl` files contain the 3D models of the respective keyboard halves and bottom plate for 3D printing.
Finally, the `matrix_pcb.kicad_pcb` file contains the generated matrix PCB for connecting the key switches.

> **Note:** The bottom plate is the same for both halves, you thus have to print it twice, ideally mirrored.

### Adding tabs to the generated matrix PCB

For manufacturing, some break-away tabs should be added to the matrix PCB. You can generate these tabs using [KiKit](https://yaqwsx.github.io/KiKit/latest/).

Start by [installing the KiKit backend](https://yaqwsx.github.io/KiKit/latest/installation/intro/) and execute the following command in the directory with the extracted exported files:

```sh
kikit panelize -p kikit_parameters.json matrix_pcb.kicad_pcb matrix_pcb_panelized.kicad_pcb
```

> **Tip**: You can also use KiKit to generate the gerber files from the KiCad file:
> ```sh
> kikit export gerber matrix_pcb_panelized.kicad_pcb matrix_pcb
> ```

## License

This project is licensed under the GNU GPLv3 license, see [`LICENSE.txt`](LICENSE.txt) for further information.
