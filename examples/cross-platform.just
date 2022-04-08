# use with https://github.com/casey/just
#
# Example cross-platform Python project
#

python_dir := if os_family() == "windows" { "./.venv/Scripts" } else { "./.venv/bin" }
python := python_dir + if os_family() == "windows" { "/python.exe" } else { "/python3" }
system_python := if os_family() == "windows" { "py.exe -3.9" } else { "python3.9" }

# Set up development environment
bootstrap:
    if test ! -e .venv; then {{ system_python }} -m venv .venv; fi
    {{ python }} -m pip install --upgrade pip wheel pip-tools
    {{ python_dir }}/pip-sync

# Upgrade Python dependencies
upgrade-deps: && bootstrap
    {{ python_dir }}/pip-compile --upgrade

# Sample project script 1
script1:
    {{ python }} script1.py

# Sample project script 2
script2 *ARGS:
    {{ python }} script2.py {{ ARGS }}
