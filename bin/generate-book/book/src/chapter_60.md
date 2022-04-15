### Node.js `package.json` Script Compatibility

The following export statement gives `just` recipes access to local Node module binaries, and makes `just` recipe commands behave more like `script` entries in Node.js `package.json` files:

````make
export PATH := "./node_modules/.bin:" + env_var('PATH')
````