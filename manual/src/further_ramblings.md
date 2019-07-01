# Further Ramblings

I personally find it very useful to write a `justfile` for almost every project, big or small.

On a big project with multiple contributors, it's very useful to have a file with all the commands needed to work on the project close at hand.

There are probably different commands to test, build, lint, deploy, and the like, and having them all in one place is useful and cuts down on the time you have to spend telling people which commands to run and how to type them.

And, with an easy place to put commands, it's likely that you'll come up with other useful things which are part of the project's collective wisdom, but which aren't written down anywhere, like the arcane commands needed for some part of your revision control workflow, install all your project's dependencies, or all the random flags you might need to pass to the build system.

Some ideas for recipes:

* Deploying/publishing the project
* Building in release mode vs debug mode
* Running in debug mode or with logging enabled
* Complex git workflows
* Updating dependencies
* Running different sets of tests, for example fast tests vs slow tests, or running them with verbose output
* Any complex set of commands that you really should write down somewhere, if only to be able to remember them

Even for small, personal projects it's nice to be able to remember commands by name instead of ^Reverse searching your shell history, and it's a huge boon to be able to go into an old project written in a random language with a mysterious build system and know that all the commands you need to do whatever you need to do are in the `justfile`, and that if you type `just` something useful (or at least interesting!) will probably happen.

For ideas for recipes, check out [this project's `justfile`](https://github.com/casey/just/blob/master/extras/justfile), or some of the `justfile`s [out in the wild](https://github.com/search?utf8=%E2%9C%93&q=filename%3Ajustfile).

I hope you enjoy using `just` and find great success and satisfaction in all your computational endeavors!

😸