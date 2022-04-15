### Pre-Built Binaries

Pre-built binaries for Linux, MacOS, and Windows can be found on [the releases page](https://github.com/casey/just/releases).

You can use the following command on Linux, MacOS, or Windows to download the latest release, just replace `DEST` with the directory where you’d like to put `just`:

````sh
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to DEST
````

For example, to install `just` to `~/bin`:

````sh
# create ~/bin
mkdir -p ~/bin

# download and extract just to ~/bin/just
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin

# add `~/bin` to the paths that your shell searches for executables
# this line should be added to your shells initialization file,
# e.g. `~/.bashrc` or `~/.zshrc`
export PATH="$PATH:$HOME/bin"

# just should now be executable
just --help
````