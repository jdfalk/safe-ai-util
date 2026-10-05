### Security

#### `allowed_patterns` is enforced, and safe-ai-util can be the only command in a sudoers rule

`CommandRestrictions.allowed_patterns` was parsed and merged but never checked,
so a `conditionally_allowed` command accepted any argument that no deny rule
caught. Every argument must now fully match an anchored pattern. A new
`allowed_argv` field pins exact argument vectors.

When safe-ai-util runs elevated (euid 0, setuid, or `SUDO_USER`/`SUDO_UID`
set), these are refused: `--config`, `--policy-overlay`, `--args-file`, and
every subcommand except the new `exec`. Elevated runs never read user or
project config or the log/audit path environment variables. `exec` reads only
`/etc/safe-ai-util/root-policy.toml`, which must be root-owned and not
group/other-writable all the way up to `/`. It runs the policy's absolute
binary with an exact argv, no shell, and an empty environment apart from a
fixed PATH. Every decision goes to syslog and a root-owned log, and the
command does not run if that log cannot be written.

`requires_elevation` now means something: the unprivileged path refuses such
rules. Two overlay widenings are closed as well. Disjoint allow-lists used to
intersect to an empty list, which read as "anything", and they now block the
command. Listing a conditional command in an overlay's `always_allowed` used
to drop its restrictions, and they are now kept. See "Using as a sudo gate"
in the README and `examples/aorg-sandbox-root-policy.toml`.

#### Root policy rules can require safe paths, and `chown -R` is a built-in

Rules can set `require_root_owned_ancestors` and `require_mount`. Before
running, the gate checks that every directory above the path is root-owned and
not group/other-writable, and that the path is the expected ZFS mount. It
matches `/proc/self/mountinfo` against an `fstat` of the path opened with
`O_NOFOLLOW|O_DIRECTORY`.

The new built-in `exec chown-tree <path>` replaces shelling out to `chown -R`,
whose symlink behaviour differs between GNU and uutils. It walks the tree with
`openat(O_NOFOLLOW|O_DIRECTORY)` and `fchownat(AT_SYMLINK_NOFOLLOW)`, and never
crosses a filesystem boundary.

In the example policy, the data clone now sets `setuid=off`, `devices=off` and
`exec=off`.
