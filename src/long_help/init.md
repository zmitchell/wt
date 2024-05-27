Creates a directory for a project and the worktree for the default branch
under it:

PROJ_NAME/
    <DEFAULT_BRANCH_NAME>

This also creates the first commit in the repository so that HEAD is defined.
Each subsequent worktree will be created as a sibling of the main worktree.
