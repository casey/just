### Stopping execution with error

Execution can be halted with the `error` function. For example:

````
foo := if "hello" == "goodbye" {
  "xyz"
} else if "a" == "b" {
  "abc"
} else {
  error("123")
}
````

Which produce the following error when run:

````
error: Call to function `error` failed: 123
   |
16 |   error("123")
````