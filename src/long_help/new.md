Creates a new worktree adjacent to the main worktree:

PROJ_NAME/
    <DEFAULT_BRANCH_NAME>/
    <NEW_WORKTREE>/

The branch associated with the worktree can either be an existing branch,
or one created for the new worktree:
- By default, when only called with the name of the worktree, a new branch with
the same name as the worktree is created.
- When called with the '-b' flag an existing branch (that is not checked out
anywhere else) will be checked out in the new worktree.
- When called with the '-n' flag a new branch with the supplied name will be
created and checked out in the new worktree.

Note that a branch can only be checked out in a single worktree, so in some
cases attempting to create a worktree will fail. For instance, if branch 'foo'
is checked out somewhere, 'wt new mywt -b foo' will fail because it will attempt
to check out the already-checked-out branch 'foo' in the new 'mywt' worktree.

Similarly, attempting to create a new worktree with 'wt new foo' will fail if
the 'foo' branch already exists since 'wt' called this way will attempt to
create a new branch 'foo' to match the name of the worktree ('foo').
