# Janus

[Janus](https://github.com/casey/janus) is a tool that collects and analyzes justfiles, and can determine if a new version of `just` breaks or changes the interpretation of existing justfiles.

Before merging a particularly large or gruesome change, Janus should be run to make sure that nothing breaks. Don't worry about running Janus yourself, Casey will happily run it for you on changes that need it.