# wt

A tool for managing a git-worktrees workflow.

This tool helps you work on projects that have this structure:
```
PROJECT_NAME/
    main/
        <files>
    foo/
        <files>
    bar/
        <files>
```
where `main/`, `foo/`, and `bar/` are worktrees for different branches.

`wt` makes it simple to set up new projects (`wt init` or `wt clone`),
create new worktrees (`wt new`), remove worktrees (`wt remove`), and list
existing worktrees (`wt list`).

See the help for each command for more details.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
