### Download and Install

First, you'll need to install the Rust toolchain (compiler, cargo, etc). Go to the [Rust website](https://www.rust-lang.org/tools/install) and follow the instructions there.

Then clone the respository recursively

`git clone --recursive https://github.com/alconley/SPShub.git`

### Testing locally
2
Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Web Locally

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

We use [Trunk](https://trunkrs.dev/) to build for web target.
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> `assets/sw.js` script will try to cache our app, and loads the cached version when it cannot connect to server allowing your app to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing us to load the latest builds during development.

### Web Deploy

My goal is to have some of the apps online for easy access. This is still under development...

1. Just run `trunk build --release`.
2. It will generate a `dist` directory as a "static html" website
3. Upload the `dist` directory to any of the numerous free hosting websites including [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
4. we already provide a workflow that auto-deploys our app to GitHub pages if you enable it.
> To enable Github Pages, you need to go to Repository -> Settings -> Pages -> Source -> set to `gh-pages` branch and `/` (root).
>
> If `gh-pages` is not available in `Source`, just create and push a branch called `gh-pages` and it should be available.

You can test the app at <https://alconley.github.io/SPShub/>.

## Apps

### Eventbuilders
- [sps_eventbuilder](https://github.com/alconley/sps_eventbuilder)
- [cebra_sps_eventbuilder](https://github.com/alconley/cebra_sps_eventbuilder)
- [cebra_eventbuilder](https://github.com/alconley/cebra_eventbuilder)

### SE-SPS Utilities
- [sps_plot](https://github.com/alconley/sps_plot)
- [sps_runtime_estimator](https://github.com/alconley/sps_runtime_estimator)

### CeBrA Utilities
- [cebra_efficiency](https://github.com/alconley/cebra_efficiency)

### General

- [muc](https://github.com/alconley/muc)
