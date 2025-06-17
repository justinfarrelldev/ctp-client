# Common Commands

This project uses cargo-make, which is available via the command line.

## Build Commands
- `cargo make build-all-windows` - Build all Windows targets (x86 and x64)
- `cargo make build-all-linux` - Build all Linux targets (x86 and x64)
- `cargo make build-all` - Build all targets for current platform
- `cargo make dev` - Build the dev server with info logging enabled

## Linting Commands
- `cargo make lint` - Run clippy linting on the whole project with strict warnings
- `cargo make lint-fix` - Run clippy with automatic fixes on the whole project

## Testing Commands
- `cargo make test` - Run the standard test suite
- `cargo make test-verbose` - Run tests with verbose output (--nocapture)
- `cargo make test-release` - Run tests in release mode for performance testing

Just before each commit, remember to run the lint-fix command. Once your current task is complete and the lint-fix command has been run, commit your changes to Git (this will check your work with the linter, tests and build commands). If the commit fails due to any issues, you must fix them - your success is contingent on the checks passing.

# Examples

You can refer to the scripts folder for an example script for how to build using cargo-make. You cannot edit anything in the scripts folder, nor the folder itself.

# Paradigms

Utilize test-driven development. You are required to maintain a minimum of 80% code coverage, as enforced in the pre-commit hook.

# Useful Information

The final (Windows) builds of the games - including all of the final game assets - are found in the ctp1 (for Civilization: Call to Power) and ctp2 (for Call to Power II) folders respectively. You are not allowed to copy ANY assets from either of these folders due to copyright, but you can use them as references for the names of various assets. You can use the source code in ctp2_source to cross-check what assets are used where and how some original logic works (as well as layouts for menus and things like that). Note that the Civilization: Call to Power source code is not available, as it is lost to time - only the Call to Power II source code is known. 

Use the original CTP2 source code to your advantage when applicable. 

# If You Encounter an Obvious Bug In The Original Source Code

If you encounter a bug in the original source code, note the change in a file called "FIXES.txt" on the top level (next to README.md) and fix the bug in the implementation within this client. For example, if the original source code for the map generation has a memory leak, do NOT include a memory leak in your implementation in this app - create an almost-equivalent implementation without the memory leak, and then note that the memory leak was noticed and will be different from the original implementation.  

If there are obvious ways to improve performance while providing an equivalent result visually, then use your more performant idea. 

# Hands Off The Following

Do not touch any of the following files or folders:
- the .git folder (including the hooks)
- this file (CLAUDE.md)
- any files in the target folder (or the folder itself)
- the scripts folder
- the Makefile.toml
- the ctp1 and ctp2 folders (do not copy anything from these, either)
- the ctp2_source folder

You may not change any lint rules, either.
