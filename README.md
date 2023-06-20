# Supertree - Supercharging `git worktree`

A small CLI meant to improve my personal workflow with git worktrees.
Whilst this could be achieved with some shell scripts, I saw this as a nice opportunity to learn and build something with Rust.


My problems with worktrees are:
1. When creating a new worktree, I wanted the new directory to have the same name as the branch.
For a branch named `feature/example`, the default behaviour of `git worktree add <name>` would be to create a branch `example` at the path `feature/example`.
The fix would be to use `git worktree add <name> -b <name>`, thus I would have to copy-paste the name always.
2. After creating a worktree, I found myself needing to copy or generate gitignored files such as env files, secret files and/or install dependencies and other tasks.

## Features

### Config file

The tool supports configuring on a project-basis what tasks to be run when creating a new worktree.
The configuration is done via a YAML config located at `${XDG_CONFIG_HOME:-$HOME/.config}/supertree/config.yaml`.
A default config will be created if missing upon running, or can be created ahead of time by running `supertree config`.

A sample config file:

```yaml
default:
  primary_branch: master
  tasks: []
projects:
  my-project:
    primary_branch: master # optional value
    primary_remote: origin # optional value
    tasks: # optional value
      - type: CopyPath
        source: node_modules
        symlink: true # optional value
      - type: Shell
        cmd: npm run prepare
      - type: CopyPath
        source: .env
        missing_okay: true # optional value
```

When creating a new worktree, the tool will try to find a project configuration in the `projects` section by using the name of the current directory as the project name.
If the name is not found under the `projects` section, the `default` section will be used.
This does not mean that the sections will be merged together.
If a specific project config is found, the `default` section is ignored altogether.

### Creating new worktrees

The simplest way is to run `supertree new <name>`, which will create a branch named `<name>` at the path `<name>`.
The new branch will be based on `master` by default (or the `primary_branch` specified in the [config file](#config-file)).

If you wish to checkout the new branch against a remote branch, you can use the `-r` flag, which will search `origin/<name>` (or your `primary_remote`) for the reference branch.
Optionally, `-r` can be supplied with a different remote branch name, like `-r=my-other-branch`.

See more via the help command - `supertree new --help`.

### Running tasks after creation

Currently 2 type of tasks can be specified:
1. `CopyPath` - used to copy or symlink a file or a directory.
The `source` field is mandatory, and together with the `primary_branch` will be used to compute the absolute source path.
That said, the tool assumes a specific directory structure for the worktree, where the `<primary_branch>` worktree is a sibling to other worktrees.
E.g. specifying to copy `node_modules` will result in the path `<project>/<primary_branch>/node_modules` to be searched.
However, if `<project_root>/<primary_branch>` does not exist, then all `CopyPath` will use `<project>` directory as the base.
The target path will be `<new_worktree_path>/<source_path>`.

A small visualization of the expected project structure:
```bash
project-example
├── new-worktree
└── master
```

2. `Shell` - run a shell command.
This command will be passed via `bash -c`, so you can use bash shenanigans if you need them.


## Nice to have future improvements
This is a work in progress tool, so it may be rough in some areas...

- Figure out how to build and share the packaged version of the tool
- Split branch name by '/' and join parts to a Path (i.e. generate platform dependant dir name)
- Pre-commit hook
- Polish logging
- Polish errors handling
    - Configure human-panic or some other alternative for better error reporting
- Support "dev" mode for the cli so that:
    - It can take a different target git repo instead of from current dir
    - It can take a different config file path instead
- Subcommand to generate autocompletion file
- Allow overriding project config with CLI config
- Clean-up created new branch and worktree object on failure


## Resources
A list of resources that I used or could use for the future changes.

- https://users.rust-lang.org/t/idiomatic-error-handling-for-resource-cleaning/26062
- https://doc.rust-lang.org/cargo/reference/profiles.html
- https://github.com/rust-cli/human-panic
- https://docs.rs/confy/0.3.1/confy/index.html
- https://rust-cli.github.io/book/tutorial/packaging.html
- https://docs.rs/clap_complete/4.3.1/clap_complete/generator/fn.generate_to.html
