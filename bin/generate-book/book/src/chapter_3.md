### Prerequisites

`just` should run on any system with a reasonable `sh`, including Linux, MacOS, and the BSDs.

On Windows, `just` works with the `sh` provided by [Git for Windows](https://git-scm.com), [GitHub Desktop](https://desktop.github.com), or [Cygwin](http://www.cygwin.com).

If you’d rather not install `sh`, you can use the `shell` setting to use the shell of your choice.

Like PowerShell:

````make
# use PowerShell instead of sh:
set shell := ["powershell.exe", "-c"]

hello:
  Write-Host "Hello, world!"
````

…or `cmd.exe`:

````make
# use cmd.exe instead of sh:
set shell := ["cmd.exe", "/c"]

list:
  dir
````

You can also set the shell using command-line arguments. For example, to use PowerShell, launch `just` with `--shell powershell.exe --shell-arg -c`.

(PowerShell is installed by default on Windows 7 SP1 and Windows Server 2008 R2 S1 and later, and `cmd.exe` is quite fiddly, so PowerShell is recommended for most Windows users.)