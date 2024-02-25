### Download and Install

First, you'll need to install the Rust toolchain (compiler, cargo, etc). Go to the [Rust website](https://www.rust-lang.org/tools/install) and follow the instructions there.

### Testing locally

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

You can test the app at <https://alconley.github.io/sps_cebra/>.

## EventBuilders

The eventbuilders were modified from [spsevb](https://github.com/gwm17/spsevb). Currently, there are 2 options for building event, one for just using the SE-SPS and the other for using the SE-SPS with CeBrA (Cerium Bromide Array).  

### Use

Enter in the information in the UI and then use the Run button.

spsevb asks the user to define a workspace. The workspace is a parent directory containing all of the relevant subdirectories for event building. When a workspace is chosen, spsevb will check to see if a) the workspace directory exists and b) if the workspace directory contains all necessary subdirectories. It will then create directories as needed (including the parent workspace directory). CoMPASS data archives (`run_<number>.tar.gz` format) should be stored in the `raw_binary` directory of the workspace. Output files (the parquet dataframe files and scaler output files) will be written to the `built` directory of the workspace.

Some important overarching notes:

- spsevb works on a run-by-run basis. That is you can specify a range of runs to event build in the UI, and spsevb will event-build and generate an output for each *individual* run. Merging runs can then be handled after the fact either through python or with a separate Rust app.

- spsevb unpacks the binary archives to the `temp_binary` directory of the workspace using the flate2 and tar crates. spsevb tries to make sure that this temporary unpacked data is always cleaned up after each run. However, in the event of a crash, sometimes `temp_binary` is not cleared. When this happens, it is a good idea to go and manually remove all binary files from `temp_binary`. spsevb should clear the directory when it starts back up, but the consequences of event building with an uncleared `temp_binary` can be severe, often making the output data illegible. Better safe than sorry.

- Make sure that you have permission to read and write to the workspace.

### Event building and the Coincidence Window

The core of event building revolves around the idea of a coincidence window. The coincidence window defines the length of time for which, after an initial detector hit, other detector hits are considered to have come from the same physics event. For spsevb, this is defined by a single user-defined value in nanoseconds, held constant for the entire event building process. spsevb uses an event building architecture similar to the [BoxScore](https://www.sciencedirect.com/science/article/abs/pii/S0168900222001954) model. The main difference is the inital sorting process: rather that using software sorting on arbitrarily buffered data, spsevb relies on the knowledge that CoMPASS saves data from each individual channel in each digitizer to its own file, and that the data in these files is already sorted in time. In a sense, CoMPASS has already done the hard work by pre-sorting so much of the data. This way, spsevb never needs to sort large data buffers, and can run a very basic modified insertion sort efficiently by merely sorting the earliest hit in time from each binary file.

A typical default value for the coincidence window is 3000 ns.

### Channel Map and Dataframe-ing

To use spsevb, there is one key component a user must create: a channel map file. The channel map provides spsevb with information linking the CAEN digitizer board/channel numbers to detector types. An example is included in the etc directory (named ChannelMap.txt). The channel map file is a three-column, whitespace delineated text file. Each row is a single channel in the entire digitizer chain. The first column indicates the board number, the second column indicates the channel number on that board, and the third column is the name of the detector component. Valid detector component names can be found in the source code in src/evb/channel_map.rs. The enum SPSChannelType has a variant for each allowed component. The variant names are the allowed component names (spelled and capitalized exactly as found in the variants). Adding new components is as simple as adding more variants to the SPSChannelType enum; all of the stringificantion and vectorization is handled by the awesome strum crate.

These channel map ids are used to link a data from a given channel to a detector component. These channel map ids are then used to generate the data fields stored in the final dataframe product. This process can be found in the source code at src/evb/sps_data.rs. There are two key components to converting to dataframe relevant structures. One is the SPSDataField enum; each variant of this enum defines one single column in the dataframe. As with the SPSChannelType enum, adding a new column is as simple as adding a new variant to SPSDataField; strum handles everything else. The other aspect is the SPSData struct. SPSData behaves much like a dictionary in Python. It contains a map of SPSDataField variants to a single 64-bit floating point value. The `new` function implemented for SPSData takes in a vector of CoMPASS data and then assigns it to an SPSDataField. This is handled by a single match statement, handling each variant of the channel map. Often times these raw detector components have three associated values (energy, energy short, and timestamp). There can also be "physics" fields, fields which are calculated using raw detector data (examples of this would be x1, x2, and xavg). These do not have an associated channel map, but are rather calculated after all raw data has been handled by checking to see if the SPSData object has identified good data from the appropriate detectors components.

### Scalers and the Scaler list

Sometimes, there are channels which contain data that should not be event built, but rather are just used as raw counting measures. A common example in the SPS setup is the beam integrator. These are commonly referred to as scalers and have to be handled slightly differently than regular data. To declare a channel a scaler, it must be added to the scaler list. The scaler list is a two column, whitespace delineated text file. The first column is the "file pattern". Since the scalers need to be declared before the event building process starts (i.e. before files are read), we cannot use the same board channel scheme used for the channel map, because CoMPASS does not name files using board numbers (which is annoying, but probably a good thing). Instead, CoMPASS names files by board serial number and channel. To that end, the file pattern is `Data_CH<channel_number>@<board_type>_<board_serial_number>`, where the fields in angle brackets should be filled out with the specific information for the scaler. The second column of the scaler list is a name for the scaler.

When a scaler is declared, spsevb removes that binary file from the list of files to event-build, and then counts the number of hits within the file. spsevb then generates a scaler output file along side the dataframe file.

### Kinematics

In brief, a first order correction to kinematic broadening of states can be done by shifting the focal plane upstream or downstream. spsevb can calculate this shift for a given reaction, specified by the target, projectile, and ejectile nuclei as well as the projectile (beam) kinetic energy, SPS (reaction) angle, and SPS magnetic field. spsevb uses this shift to calculate "weights" to apply to the data from the front and back delay lines. The weights are factors equivalent to finding the solution of tracing the particle trajectory to the shifted focal plane. For more information, see the papers by H. Enge on the Enge splipole designs.

In spsevb, nuclei are specified by Z, A. The residual is calculated from the other nuclei. Beam kinetic energy is given in MeV, angle in degrees, and magnetic field in kG (Tesla). Nuclear data is retrieved from the [AMDC](https://www-nds.iaea.org/amdc/) 2016 mass file distributed with the repository. Since the program has the path to the AMDC file hardcoded, always run from the top level of the repository.

The Set button of the kinematics section should be renamed. It does not set values, merely sets the reaction equation.

### Memory Usage and Max Buffer Size

Once data is event built, it is stored in a map like structure which is stored on the heap until converted to a dataframe and written to disk. This does mean that spsevb will need to store the entire dataset in memory (a buffer) until it is written to disk. In general this is a benefit; all file writing occurs at once, which allows the event building to proceed as quickly as possible. However, this can mean that once progress has reached 100%, the progress may "freeze" for a second before allowing a new run command, as writing data to disk can take some time.

As a precaution against extremely large single run datasets, spsevb has a limit on the maximum size of a buffer as 8GB by default. Once the limit is reached, spsevb will stop event building, convert the data and write to disk, and then resume event building. When this fragmentation happens, the spsevb will append a fragment number to the output file name (i.e. `run_<run_num>_<frag_num>.parquet`). These fragment files can be combined later if needed (though in general this is not recommended). Most SPS experiments should never reach this limit, but it is a necessary precaution. This limit may need to be adjusted depending on the hardware used (the max buffer size should not exceed system memory).

Currently max file size is defined in `src/compass_run.rs` as a constant. Eventually this will be promoted to an user input in the GUI.

### Configuration saving

The File menu has options for saving and loading configurations. Configurations are stored as YAML files (using the serde and serde_yaml crates), which are human readable and editable.


## SPS Plot

This tool is intended to be used for guiding the settings of the SPS to show specific states on the focal plane detector. The user gives the program reaction information, and the program runs through the kinematics to calculate the energies of ejecta into the the SESPS. To evaluate different states, the program scrapes a list of levels from NNDC, and these levels are then passed on to the reaction handler. These levels are then shown on the screen with labels. The labels can be modified to show either the excitation energy of the state, the kinetic energy of the ejectile, or the focal plane z-offset for a state. Note that since levels are obtained from NNDC, SPSPlot requires an internet connection.

This tool is a simplier version of a tool located in [SPSPy](https://github.com/gwm17/spspy) and written in rust.