Removes the specified worktree(s).

The branches checked out in each worktree are also deleted by default, but you
may leave the branches intact with the `-l/--leave-branches` option.

When no worktrees are specified the user will be presented with a prompt to
select the worktrees to remove. The main worktree is *never* included in this
list, so if you want to delete all worktrees except the main one you can simply
press `->` to select all worktrees and remove them without worry.

You will be prompted to confirm that you want to delete the specified worktrees
unless the `-f/--force` option is specified.
