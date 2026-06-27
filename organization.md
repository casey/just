Organization
============

## Proposed Grouping

*Italic* = pulled up from `Miscellanea`.

### Recipes
- The Default Recipe
- Recipe Parameters (incl. Recipe Flags and Options)
- Avoiding Argument Splitting (Quoting / Positional Arguments / Exported Arguments)
- Dependencies (incl. Running Recipes at the End / in the Middle)
- *Parallelism*
- Documentation Comments
- Groups
- Aliases
- Private Recipes
- Enabling and Disabling Recipes

### Expressions
- Variables and Assignments
- Expressions and Substitutions (Concatenation / Logical Operators / Joining Paths / Escaping `{{`)
- Strings (Shell-expanded strings / Format strings)
- Conditional Expressions
- Command Evaluation Using Backticks
- Stopping execution with error
- Built-in Functions (everything but the list of functions)
- User-defined functions

### Execution
- Sigils
- Quiet Recipes
- Shebang Recipes
- Script Recipes
- Script and Shebang Recipe Temporary Files
- Safer Bash Shebang Recipes (incl. Shebang Recipe Execution on Windows)
- Indentation
- Multi-Line Constructs (if / for / while / Outside Recipe Bodies)
- Setting Variables in a Recipe
- Configuring the Shell

### Environment Variables
- Getting and Setting Environment Variables (Exporting / Unexporting / Getting / Setting from Env)
- Sharing Environment Variables Between Recipes (Using Python Virtual Environments)

### Working Directory
- Working Directory
- Changing the Working Directory in a Recipe
- Disabling Changing Directory

### Organization
- Imports
- Modules
- Invoking `justfile`s in Other Directories
- Fallback to parent `justfile`s
- *Remote Justfiles*
- *Global and User `justfile`s*
- Markdown `justfile`s
- Just Scripts
- Hiding `justfile`s

### Command Line
- Listing Available Recipes
- Invoking Multiple Recipes
- Setting Variables from the Command Line
- Command-line Options (incl. Setting Command-line Options with Environment Variables)
- Selecting Recipes to Run With an Interactive Chooser
- Requiring Confirmation for Recipes (incl. Custom Confirmation Prompt)
- Timestamps
- Signal Handling (Fatal Signals / Continuing Execution / SIGINFO / Windows)
- *Shell Completion Scripts*
- *Man Page*
- Formatting and dumping `justfile`s

### Cached Recipes
- Cached Recipes (Clearing the Cache / Input Files / Output Files / Friendly Admonitions)

### Reference
- Attributes (the table of attributes)
- Settings (Table of Settings + all per-setting subsections)
- Build-in functions (the lists of functions)
- Constants

### Miscellanea (not exhuastive, just everything that didn't get moved)
Shell Alias · Grammar · just.sh · Node.js `package.json` Script Compatibility ·
Paths on Windows · Printing Complex Strings · Skill for Agents · Alternatives and
Prior Art, Python Recipes with `uv`, Activating Environments, Metadata
*Re-running recipes when files change*
