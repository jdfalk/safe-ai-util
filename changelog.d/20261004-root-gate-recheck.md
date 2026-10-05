### Security

#### Root-gate hardening from the second security review

- **Rule shadowing:** every rule for a binary is consulted, however the
  command is invoked. The preconditions of every rule that accepts the argv
  apply, so a broad rule can no longer shadow a guarded one. A policy that
  lists the same exact argv in two rules for one binary is rejected, and so
  is one that configures the same `chown_trees` path twice.
- **`chown-tree` uses one descriptor:** it opens the target without
  following symlinks and checks the ancestors, ZFS `fstatfs` magic, the
  mountinfo source, mount root `/` and the device, all on the descriptor it
  then walks.
- **`chown-tree` walk:**
  - It runs post-order, so the top directory changes last.
  - Non-directories are changed through an `O_PATH|O_NOFOLLOW` descriptor
    with `AT_EMPTY_PATH`.
  - `readdir` errors are distinguished from end-of-directory.
  - Depth is capped, and `RLIMIT_NOFILE` is raised first.
  - Per-entry failures are counted, and the run exits 1 if any occur.
  - A top-level `.zfs` is skipped.
- **Mountinfo parsing:** out-of-range octal escapes are rejected rather than
  overflowing, and `major`/`minor` use glibc's masks.
- **Child process and log:** the child runs with umask 022. The audit-log
  directory is resolved without following symlinks in any component.
- **Example policy:** the data clone adds `snapdir=hidden`, `sharenfs=off`
  and `sharesmb=off`.
- **Docs:** the README warns against sudo's `closefrom_override`.
