### Dotenv Integration

If [`dotenv-load`](#dotenv-load) is set, `just` will load environment variables from a file named `.env`. This file can be located in the same directory as your `justfile` or in a parent directory. These variables are environment variables, not `just` variables, and so must be accessed using `$VARIABLE_NAME` in recipes and backticks.

For example, if your `.env` file contains:

````sh
# a comment, will be ignored
DATABASE_ADDRESS=localhost:6379
SERVER_PORT=1337
````

And your `justfile` contains:

````make
set dotenv-load

serve:
  @echo "Starting server with database $DATABASE_ADDRESS on port $SERVER_PORT…"
  ./server --database $DATABASE_ADDRESS --port $SERVER_PORT
````

`just serve` will output:

````sh
$ just serve
Starting server with database localhost:6379 on port 1337…
./server --database $DATABASE_ADDRESS --port $SERVER_PORT
````