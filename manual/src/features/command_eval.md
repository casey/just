# Command Evaluation Using Backticks

Backticks can be used to store the result of commands:

```make
localhost := `dumpinterfaces | cut -d: -f2 | sed 's/\/.*//' | sed 's/ //g'`

serve:
    ./serve {{localhost}} 8080
```
