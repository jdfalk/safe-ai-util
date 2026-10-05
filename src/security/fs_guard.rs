// file: src/security/fs_guard.rs
// version: 1.1.0
// guid: 3a7e1c95-6d2b-4f08-8c4e-b1f9d0a26e73
// last-edited: 2026-10-04

//! Filesystem preconditions and the built-in `chown-tree` action.
//!
//! The root policy pins argv, but argv names paths, and a path is only as
//! trustworthy as the directories above it. This module checks two things
//! before an elevated action touches a path:
//!
//! * **Root-owned ancestors.** Every directory above the target, from `/`
//!   down to the target's parent, must be a real directory (not a symlink),
//!   owned by root and not writable by group or other. Otherwise an
//!   unprivileged user could swap a component and redirect the action.
//! * **Expected mount.** The target must be the mount point of a specific ZFS
//!   dataset. This is checked by matching `/proc/self/mountinfo` against an
//!   `fstat` of the target opened with `O_NOFOLLOW|O_DIRECTORY`, so the
//!   directory that is checked is the directory that will be used.
//!
//! `chown-tree` replaces shelling out to `chown -R`. It never follows a
//! symlink (`openat(O_NOFOLLOW|O_DIRECTORY)` for every directory,
//! `fchownat(AT_SYMLINK_NOFOLLOW)` for everything else) and never leaves the
//! target's filesystem (`st_dev` is compared on every entry). Because hard
//! links cannot span filesystems, nothing outside the target dataset can be
//! reached through the walk, whatever the tree contains.
//!
//! Both checks go through [`FsView`] so tests can simulate other owners and
//! mount tables without root.

use std::io;
use std::path::{Path, PathBuf};

/// The parts of `stat` the checks use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeInfo {
    pub uid: u32,
    pub mode: u32,
    pub dev: u64,
    pub is_dir: bool,
    pub is_symlink: bool,
}

/// Read-only view of the filesystem used by the precondition checks.
pub trait FsView {
    /// `lstat` (does not follow a final symlink).
    fn lstat(&self, path: &Path) -> io::Result<NodeInfo>;
    /// Open `path` with `O_NOFOLLOW|O_DIRECTORY` and `fstat` the descriptor.
    fn dir_fstat(&self, path: &Path) -> io::Result<NodeInfo>;
    /// Contents of `/proc/self/mountinfo`.
    fn mountinfo(&self) -> io::Result<String>;
}

/// The real filesystem.
pub struct RealFs;

#[cfg(unix)]
fn node_info(meta: &std::fs::Metadata) -> NodeInfo {
    use std::os::unix::fs::MetadataExt;
    NodeInfo {
        uid: meta.uid(),
        mode: meta.mode(),
        dev: meta.dev(),
        is_dir: meta.file_type().is_dir(),
        is_symlink: meta.file_type().is_symlink(),
    }
}

#[cfg(unix)]
impl FsView for RealFs {
    fn lstat(&self, path: &Path) -> io::Result<NodeInfo> {
        Ok(node_info(&std::fs::symlink_metadata(path)?))
    }

    fn dir_fstat(&self, path: &Path) -> io::Result<NodeInfo> {
        use std::os::unix::fs::OpenOptionsExt;
        let f = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
            .open(path)?;
        Ok(node_info(&f.metadata()?))
    }

    fn mountinfo(&self) -> io::Result<String> {
        std::fs::read_to_string("/proc/self/mountinfo")
    }
}

#[cfg(not(unix))]
impl FsView for RealFs {
    fn lstat(&self, _path: &Path) -> io::Result<NodeInfo> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "unix only"))
    }
    fn dir_fstat(&self, _path: &Path) -> io::Result<NodeInfo> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "unix only"))
    }
    fn mountinfo(&self) -> io::Result<String> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "unix only"))
    }
}

fn denied(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, msg)
}

/// True for an absolute path spelled canonically: starts with `/`, no empty,
/// `.` or `..` segments, no trailing slash (except `/` itself). Checked on the
/// raw string because `Path::components` silently drops `.` and `//`.
pub fn is_clean_absolute(p: &Path) -> bool {
    let Some(s) = p.to_str() else {
        return false;
    };
    if s == "/" {
        return true;
    }
    let Some(rest) = s.strip_prefix('/') else {
        return false;
    };
    rest.split('/')
        .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

/// Every proper ancestor of `path`, from `/` down to its parent.
fn ancestors_top_down(path: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = path.ancestors().skip(1).map(Path::to_path_buf).collect();
    v.reverse();
    v
}

/// Check that every ancestor of `path` is a real directory owned by one of
/// `trusted_uids` (production: root only) and not writable by group or
/// other. The target itself is not checked; it may not exist yet (a clone's
/// future mount point) or may be owned by the sandbox user after a previous
/// `chown-tree`.
pub fn check_root_owned_ancestors(
    fs: &dyn FsView,
    path: &Path,
    trusted_uids: &[u32],
) -> io::Result<()> {
    if !is_clean_absolute(path) || path.parent().is_none() {
        return Err(denied(format!(
            "{} must be an absolute path with no '.' or '..' and a parent",
            path.display()
        )));
    }
    for dir in ancestors_top_down(path) {
        let n = fs.lstat(&dir).map_err(|e| {
            denied(format!(
                "ancestor {} cannot be checked: {}",
                dir.display(),
                e
            ))
        })?;
        if n.is_symlink || !n.is_dir {
            return Err(denied(format!(
                "ancestor {} is not a real directory",
                dir.display()
            )));
        }
        if !trusted_uids.contains(&n.uid) {
            return Err(denied(format!(
                "ancestor {} is owned by uid {}, not root",
                dir.display(),
                n.uid
            )));
        }
        if n.mode & 0o022 != 0 {
            return Err(denied(format!(
                "ancestor {} is writable by group or other (mode {:o})",
                dir.display(),
                n.mode & 0o7777
            )));
        }
    }
    Ok(())
}

/// One line of `/proc/self/mountinfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountEntry {
    pub major: u64,
    pub minor: u64,
    /// Root of the mount within its filesystem (`/` unless a bind mount of a
    /// subdirectory).
    pub root: String,
    pub mount_point: String,
    pub fstype: String,
    pub source: String,
}

/// Undo mountinfo's octal escapes (`\040` for space and friends). Returns
/// `None` for an escape above `\377`, which the kernel never writes.
fn unescape_mountinfo(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\'
            && b.len() >= i + 4
            && b[i + 1..i + 4].iter().all(|c| (b'0'..=b'7').contains(c))
        {
            let v = u32::from(b[i + 1] - b'0') * 64
                + u32::from(b[i + 2] - b'0') * 8
                + u32::from(b[i + 3] - b'0');
            out.push(u8::try_from(v).ok()?);
            i += 4;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    Some(String::from_utf8_lossy(&out).into_owned())
}

/// Parse mountinfo text. Malformed lines are skipped (they cannot match).
pub fn parse_mountinfo(text: &str) -> Vec<MountEntry> {
    let mut out = Vec::new();
    for line in text.lines() {
        let fields: Vec<&str> = line.split(' ').collect();
        // id parent major:minor root mount_point opts [optional...] - fstype source superopts
        let Some(sep) = fields.iter().position(|f| *f == "-") else {
            continue;
        };
        if fields.len() < 6 || fields.len() < sep + 3 {
            continue;
        }
        let Some((maj, min)) = fields[2].split_once(':') else {
            continue;
        };
        let (Ok(major), Ok(minor)) = (maj.parse(), min.parse()) else {
            continue;
        };
        let (Some(root), Some(mount_point), Some(source)) = (
            unescape_mountinfo(fields[3]),
            unescape_mountinfo(fields[4]),
            unescape_mountinfo(fields[sep + 2]),
        ) else {
            continue;
        };
        out.push(MountEntry {
            major,
            minor,
            root,
            mount_point,
            fstype: fields[sep + 1].to_string(),
            source,
        });
    }
    out
}

/// glibc's `gnu_dev_major`/`gnu_dev_minor` for a 64-bit `dev_t`.
pub fn dev_major_minor(dev: u64) -> (u64, u64) {
    let major = ((dev >> 8) & 0x0000_0fff) | ((dev >> 32) & 0xffff_f000);
    let minor = (dev & 0x0000_00ff) | ((dev >> 12) & 0xffff_ff00);
    (major, minor)
}

/// Match a mount table against an already-opened directory's device id.
///
/// The topmost mount at `path` (the last matching line) must be `zfs`, have
/// source `dataset`, have root `/` (not a bind mount of a subdirectory of
/// some other dataset), and its `major:minor` must equal `dev`.
pub fn match_mount(mountinfo: &str, path: &Path, dataset: &str, dev: u64) -> io::Result<()> {
    let want = path.to_string_lossy();
    let entry = parse_mountinfo(mountinfo)
        .into_iter()
        .rev()
        .find(|m| m.mount_point == want)
        .ok_or_else(|| denied(format!("nothing is mounted at {}", path.display())))?;
    if entry.fstype != "zfs" || entry.source != dataset {
        return Err(denied(format!(
            "{} is {} {}, expected zfs {}",
            path.display(),
            entry.fstype,
            entry.source,
            dataset
        )));
    }
    if entry.root != "/" {
        return Err(denied(format!(
            "{} is a bind mount of {} within {}, not the dataset root",
            path.display(),
            entry.root,
            dataset
        )));
    }
    if dev_major_minor(dev) != (entry.major, entry.minor) {
        return Err(denied(format!(
            "{} device {:?} does not match the {} mount {}:{}",
            path.display(),
            dev_major_minor(dev),
            dataset,
            entry.major,
            entry.minor
        )));
    }
    Ok(())
}

/// Check that `path` is currently the mount point of ZFS dataset `dataset`,
/// and return the device id of that mount. Used for preconditions and dry
/// runs; `chown-tree` repeats the check on the descriptor it walks.
pub fn check_mount_of(fs: &dyn FsView, path: &Path, dataset: &str) -> io::Result<u64> {
    let node = fs.dir_fstat(path).map_err(|e| {
        denied(format!(
            "{} cannot be opened as a directory: {}",
            path.display(),
            e
        ))
    })?;
    let text = fs
        .mountinfo()
        .map_err(|e| denied(format!("cannot read mountinfo: {}", e)))?;
    match_mount(&text, path, dataset, node.dev)?;
    Ok(node.dev)
}

/// Result of a `chown-tree` walk.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChownStats {
    /// Entries whose ownership was set (including the top directory).
    pub changed: u64,
    /// Entries skipped because they are on another filesystem (nested mounts).
    pub skipped_other_dev: u64,
    /// Entries that could not be changed or read (the walk carries on).
    pub failed: u64,
    /// Directories not entered because they were deeper than [`MAX_DEPTH`].
    pub too_deep: u64,
}

/// Deepest directory level `chown-tree` enters. One descriptor is held per
/// level, so this also bounds descriptor use.
pub const MAX_DEPTH: usize = 256;

/// `ZFS_SUPER_MAGIC` from OpenZFS (`f_type` reported by `fstatfs`).
pub const ZFS_SUPER_MAGIC: u64 = 0x2fc1_2fc1;

#[cfg(unix)]
mod walk {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::fs::File;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::{AsRawFd, FromRawFd};

    const DIR_FLAGS: libc::c_int =
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;

    fn cstr(b: &[u8]) -> io::Result<CString> {
        CString::new(b).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in path"))
    }

    fn openat_dir(dirfd: libc::c_int, name: &CStr) -> io::Result<File> {
        // SAFETY: name is NUL-terminated; dirfd is open for the call; the
        // returned descriptor is owned by the File.
        let fd = unsafe { libc::openat(dirfd, name.as_ptr(), DIR_FLAGS) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: fd was just opened and is owned by nobody else.
        Ok(unsafe { File::from_raw_fd(fd) })
    }

    fn fstat_fd(fd: libc::c_int) -> io::Result<libc::stat> {
        // SAFETY: st is fully written by a successful fstat.
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(fd, &mut st) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(st)
    }

    fn fstat(f: &File) -> io::Result<libc::stat> {
        fstat_fd(f.as_raw_fd())
    }

    #[cfg(not(target_os = "linux"))]
    fn fstatat_nofollow(dirfd: libc::c_int, name: &CStr) -> io::Result<libc::stat> {
        // SAFETY: as above; AT_SYMLINK_NOFOLLOW stats a symlink itself.
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstatat(dirfd, name.as_ptr(), &mut st, libc::AT_SYMLINK_NOFOLLOW) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(st)
    }

    // The casts below are identities on Linux but not elsewhere: mode_t is
    // u16 and dev_t is i32 on macOS. Keep them in one place.
    #[allow(clippy::unnecessary_cast)]
    fn st_mode(st: &libc::stat) -> u32 {
        st.st_mode as u32
    }

    #[allow(clippy::unnecessary_cast)]
    fn st_dev(st: &libc::stat) -> u64 {
        st.st_dev as u64
    }

    #[allow(clippy::unnecessary_cast)]
    fn is_dir(st: &libc::stat) -> bool {
        (st_mode(st) & libc::S_IFMT as u32) == libc::S_IFDIR as u32
    }

    #[cfg(target_os = "linux")]
    fn clear_errno() {
        // SAFETY: __errno_location returns this thread's errno slot.
        unsafe { *libc::__errno_location() = 0 }
    }

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
    fn clear_errno() {
        // SAFETY: __error returns this thread's errno slot.
        unsafe { *libc::__error() = 0 }
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    )))]
    fn clear_errno() {}

    /// Raise the soft open-file limit to the hard limit (best effort).
    pub fn raise_nofile_limit() {
        // SAFETY: getrlimit/setrlimit only read and write the struct.
        unsafe {
            let mut rl: libc::rlimit = std::mem::zeroed();
            if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rl) == 0 && rl.rlim_cur < rl.rlim_max {
                rl.rlim_cur = rl.rlim_max;
                let _ = libc::setrlimit(libc::RLIMIT_NOFILE, &rl);
            }
        }
    }

    /// Walk `path` one component at a time from `/` with
    /// `openat(O_NOFOLLOW|O_DIRECTORY)`. A symlink in any component fails.
    /// When `trusted_uids` is given, every directory opened above the target
    /// is also checked through its descriptor (owner trusted, not
    /// group/other-writable).
    pub fn open_dir_chain(path: &Path, trusted_uids: Option<&[u32]>) -> io::Result<File> {
        if !is_clean_absolute(path) {
            return Err(denied(format!(
                "{} is not a clean absolute path",
                path.display()
            )));
        }
        let root = cstr(b"/")?;
        // SAFETY: as in openat_dir.
        let fd = unsafe { libc::open(root.as_ptr(), DIR_FLAGS) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: fd was just opened.
        let mut cur = unsafe { File::from_raw_fd(fd) };
        let mut shown = PathBuf::from("/");
        for comp in path.components().skip(1) {
            if let Some(trusted) = trusted_uids {
                let st = fstat(&cur)?;
                if !trusted.contains(&st.st_uid) || st_mode(&st) & 0o022 != 0 {
                    return Err(denied(format!(
                        "ancestor {} is not root-owned or is writable by group or other",
                        shown.display()
                    )));
                }
            }
            let name = cstr(comp.as_os_str().as_bytes())?;
            cur = openat_dir(cur.as_raw_fd(), &name).map_err(|e| {
                denied(format!(
                    "{} cannot be opened without following symlinks: {}",
                    shown.join(comp).display(),
                    e
                ))
            })?;
            shown.push(comp);
        }
        Ok(cur)
    }

    /// [`open_dir_chain`] with the ancestor ownership checks.
    pub fn open_beneath_root(path: &Path, trusted_uids: &[u32]) -> io::Result<File> {
        open_dir_chain(path, Some(trusted_uids))
    }

    /// `f_type` of the filesystem holding `f`.
    #[cfg(target_os = "linux")]
    #[allow(clippy::unnecessary_cast)]
    pub fn fs_magic(f: &File) -> io::Result<u64> {
        // SAFETY: sfs is fully written by a successful fstatfs.
        let mut sfs: libc::statfs = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstatfs(f.as_raw_fd(), &mut sfs) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(sfs.f_type as u64)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn fs_magic(_f: &File) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "filesystem magic check is Linux-only",
        ))
    }

    /// List entry names of an open directory (excluding `.` and `..`).
    fn read_names(dir: &File) -> io::Result<Vec<CString>> {
        // SAFETY: dup gives fdopendir its own descriptor; closedir closes it.
        let dupfd = unsafe { libc::fcntl(dir.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 0) };
        if dupfd < 0 {
            return Err(io::Error::last_os_error());
        }
        let dp = unsafe { libc::fdopendir(dupfd) };
        if dp.is_null() {
            let e = io::Error::last_os_error();
            unsafe { libc::close(dupfd) };
            return Err(e);
        }
        unsafe { libc::rewinddir(dp) };
        let mut names = Vec::new();
        let result = loop {
            // readdir returns NULL both at the end and on error; only errno
            // tells them apart, so clear it first.
            clear_errno();
            // SAFETY: dp is a valid DIR*; the returned entry is valid until
            // the next readdir/closedir, and we copy the name out at once.
            let ent = unsafe { libc::readdir(dp) };
            if ent.is_null() {
                let e = io::Error::last_os_error();
                break match e.raw_os_error() {
                    Some(0) | None => Ok(()),
                    Some(_) => Err(e),
                };
            }
            let name = unsafe { CStr::from_ptr((*ent).d_name.as_ptr()) };
            let b = name.to_bytes();
            if b != b"." && b != b".." {
                names.push(name.to_owned());
            }
        };
        unsafe { libc::closedir(dp) };
        result.map(|()| names)
    }

    /// Change ownership of one non-directory entry without following it.
    ///
    /// Linux: open it `O_PATH|O_NOFOLLOW`, `fstat` the descriptor (device
    /// and type), then `fchownat(fd, "", AT_EMPTY_PATH)`, so the checked
    /// inode is the changed inode. Elsewhere: `fstatat` + `fchownat` with
    /// `AT_SYMLINK_NOFOLLOW`.
    /// Returns `Ok(false)` when skipped (other device, became a directory,
    /// or vanished).
    #[cfg(target_os = "linux")]
    fn chown_leaf(
        dirfd: libc::c_int,
        name: &CStr,
        dev: u64,
        uid: u32,
        gid: u32,
    ) -> io::Result<Leaf> {
        // SAFETY: name is NUL-terminated; the fd is closed by `File`.
        let fd = unsafe {
            libc::openat(
                dirfd,
                name.as_ptr(),
                libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            let e = io::Error::last_os_error();
            return if e.raw_os_error() == Some(libc::ENOENT) {
                Ok(Leaf::Gone)
            } else {
                Err(e)
            };
        }
        // SAFETY: fd was just opened.
        let f = unsafe { File::from_raw_fd(fd) };
        let st = fstat(&f)?;
        if st_dev(&st) != dev {
            return Ok(Leaf::OtherDev);
        }
        if is_dir(&st) {
            return Ok(Leaf::IsDir);
        }
        let empty = cstr(b"")?;
        // SAFETY: f's fd is open; AT_EMPTY_PATH acts on the fd itself.
        let rc = unsafe {
            libc::fchownat(
                f.as_raw_fd(),
                empty.as_ptr(),
                uid,
                gid,
                libc::AT_EMPTY_PATH | libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if rc != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Leaf::Changed)
    }

    #[cfg(not(target_os = "linux"))]
    fn chown_leaf(
        dirfd: libc::c_int,
        name: &CStr,
        dev: u64,
        uid: u32,
        gid: u32,
    ) -> io::Result<Leaf> {
        let st = match fstatat_nofollow(dirfd, name) {
            Ok(st) => st,
            Err(e) if e.raw_os_error() == Some(libc::ENOENT) => return Ok(Leaf::Gone),
            Err(e) => return Err(e),
        };
        if st_dev(&st) != dev {
            return Ok(Leaf::OtherDev);
        }
        if is_dir(&st) {
            return Ok(Leaf::IsDir);
        }
        // SAFETY: dirfd is open; name is NUL-terminated.
        let rc =
            unsafe { libc::fchownat(dirfd, name.as_ptr(), uid, gid, libc::AT_SYMLINK_NOFOLLOW) };
        if rc != 0 {
            let e = io::Error::last_os_error();
            return if e.raw_os_error() == Some(libc::ENOENT) {
                Ok(Leaf::Gone)
            } else {
                Err(e)
            };
        }
        Ok(Leaf::Changed)
    }

    enum Leaf {
        Changed,
        Gone,
        OtherDev,
        IsDir,
    }

    struct Frame {
        dir: File,
        names: Vec<CString>,
        next: usize,
    }

    /// Set `uid:gid` on `top` and everything beneath it on the same
    /// filesystem (`dev`), never following symlinks, in post-order: a
    /// directory's contents are changed before the directory itself, and
    /// `top` is changed last. Per-entry failures are counted and the walk
    /// carries on.
    pub fn chown_tree_fd(top: File, dev: u64, uid: u32, gid: u32) -> io::Result<ChownStats> {
        let mut stats = ChownStats::default();
        let st = fstat(&top)?;
        if st_dev(&st) != dev || !is_dir(&st) {
            return Err(denied("top of tree is not the expected directory".into()));
        }
        let names = read_names(&top)?;
        let mut stack = vec![Frame {
            dir: top,
            names,
            next: 0,
        }];
        while let Some(frame) = stack.last_mut() {
            if frame.next == frame.names.len() {
                let done = stack.pop().expect("non-empty");
                // SAFETY: fd is open; fchown on a descriptor cannot be
                // redirected.
                if unsafe { libc::fchown(done.dir.as_raw_fd(), uid, gid) } != 0 {
                    stats.failed += 1;
                } else {
                    stats.changed += 1;
                }
                continue;
            }
            let name = frame.names[frame.next].clone();
            frame.next += 1;
            let at_top = stack.len() == 1;
            // A visible .zfs control directory at the dataset root holds
            // snapshots (automounted, other devices); never descend into it.
            if at_top && name.as_bytes() == b".zfs" {
                stats.skipped_other_dev += 1;
                continue;
            }
            let dirfd = stack.last().expect("non-empty").dir.as_raw_fd();
            match chown_leaf(dirfd, &name, dev, uid, gid) {
                Ok(Leaf::Changed) => stats.changed += 1,
                Ok(Leaf::Gone) => {}
                Ok(Leaf::OtherDev) => stats.skipped_other_dev += 1,
                Err(_) => stats.failed += 1,
                Ok(Leaf::IsDir) => {
                    if stack.len() >= MAX_DEPTH {
                        stats.too_deep += 1;
                        continue;
                    }
                    let sub = match openat_dir(dirfd, &name) {
                        Ok(f) => f,
                        // Removed, or swapped for a symlink or a file since
                        // it was checked; O_NOFOLLOW refuses to follow.
                        Err(e)
                            if matches!(
                                e.raw_os_error(),
                                Some(libc::ENOENT) | Some(libc::ELOOP) | Some(libc::ENOTDIR)
                            ) =>
                        {
                            continue
                        }
                        Err(_) => {
                            stats.failed += 1;
                            continue;
                        }
                    };
                    match fstat(&sub) {
                        Ok(sst) if st_dev(&sst) == dev && is_dir(&sst) => {}
                        Ok(_) => {
                            stats.skipped_other_dev += 1;
                            continue;
                        }
                        Err(_) => {
                            stats.failed += 1;
                            continue;
                        }
                    }
                    match read_names(&sub) {
                        Ok(names) => stack.push(Frame {
                            dir: sub,
                            names,
                            next: 0,
                        }),
                        Err(_) => {
                            // Still change the directory itself.
                            stats.failed += 1;
                            // SAFETY: as above.
                            if unsafe { libc::fchown(sub.as_raw_fd(), uid, gid) } == 0 {
                                stats.changed += 1;
                            }
                        }
                    }
                }
            }
        }
        Ok(stats)
    }
}

#[cfg(unix)]
pub use walk::{chown_tree_fd, open_beneath_root, open_dir_chain};

/// Run a whole `chown-tree` on one descriptor:
///
/// 1. open `path` from `/` without following symlinks, checking every
///    ancestor's owner and mode on the descriptors actually opened;
/// 2. on that descriptor: `fstat` (device), `fstatfs` (ZFS magic), and the
///    mountinfo match for `dataset` (root `/`, matching `major:minor`);
/// 3. raise `RLIMIT_NOFILE` and walk that same descriptor.
#[cfg(unix)]
pub fn chown_tree(
    path: &Path,
    dataset: &str,
    uid: u32,
    gid: u32,
    trusted_uids: &[u32],
    mountinfo: &dyn Fn() -> io::Result<String>,
) -> io::Result<ChownStats> {
    use std::os::unix::fs::MetadataExt;
    let top = open_beneath_root(path, trusted_uids)?;
    let dev = top.metadata()?.dev();
    let magic = walk::fs_magic(&top)?;
    if magic != ZFS_SUPER_MAGIC {
        return Err(denied(format!(
            "{} is not on ZFS (f_type {:#x})",
            path.display(),
            magic
        )));
    }
    let text = mountinfo().map_err(|e| denied(format!("cannot read mountinfo: {}", e)))?;
    match_mount(&text, path, dataset, dev)?;
    walk::raise_nofile_limit();
    chown_tree_fd(top, dev, uid, gid)
}

#[cfg(not(unix))]
pub fn chown_tree(
    _path: &Path,
    _dataset: &str,
    _uid: u32,
    _gid: u32,
    _trusted_uids: &[u32],
    _mountinfo: &dyn Fn() -> io::Result<String>,
) -> io::Result<ChownStats> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "unix only"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// A fake filesystem: path -> node, plus a mountinfo text.
    struct FakeFs {
        nodes: HashMap<PathBuf, NodeInfo>,
        mountinfo: String,
    }

    fn dir(uid: u32, mode: u32, dev: u64) -> NodeInfo {
        NodeInfo {
            uid,
            mode,
            dev,
            is_dir: true,
            is_symlink: false,
        }
    }

    impl FakeFs {
        fn sandbox() -> Self {
            let mut nodes = HashMap::new();
            nodes.insert(PathBuf::from("/"), dir(0, 0o755, 1));
            nodes.insert(PathBuf::from("/mnt"), dir(0, 0o755, 1));
            nodes.insert(PathBuf::from("/mnt/aorg-sandbox"), dir(0, 0o755, 1));
            // st_dev 0:57 == makedev(0, 57) == 57
            nodes.insert(
                PathBuf::from("/mnt/aorg-sandbox/data"),
                dir(1000, 0o755, 57),
            );
            let mountinfo = "\
22 1 0:21 / / rw,relatime shared:1 - zfs rpool/ROOT rw,xattr,posixacl
95 22 0:57 / /mnt/aorg-sandbox/data rw,nosuid,nodev,noexec,relatime shared:50 - zfs bigdata/rehearsal-sandbox-data rw,xattr,posixacl
"
            .to_string();
            Self { nodes, mountinfo }
        }
    }

    impl FsView for FakeFs {
        fn lstat(&self, path: &Path) -> io::Result<NodeInfo> {
            self.nodes
                .get(path)
                .copied()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such node"))
        }
        fn dir_fstat(&self, path: &Path) -> io::Result<NodeInfo> {
            let n = self.lstat(path)?;
            if n.is_symlink {
                return Err(io::Error::from_raw_os_error(40)); // ELOOP
            }
            Ok(n)
        }
        fn mountinfo(&self) -> io::Result<String> {
            Ok(self.mountinfo.clone())
        }
    }

    const TARGET: &str = "/mnt/aorg-sandbox/data";

    #[test]
    fn ancestors_ok_on_a_root_owned_chain() {
        let fs = FakeFs::sandbox();
        check_root_owned_ancestors(&fs, Path::new(TARGET), &[0]).unwrap();
    }

    #[test]
    fn ancestor_owned_by_non_root_is_refused() {
        let mut fs = FakeFs::sandbox();
        fs.nodes
            .insert(PathBuf::from("/mnt/aorg-sandbox"), dir(1000, 0o755, 1));
        let err = check_root_owned_ancestors(&fs, Path::new(TARGET), &[0]).unwrap_err();
        assert!(err.to_string().contains("owned by uid 1000"), "{err}");
    }

    #[test]
    fn ancestor_writable_by_group_or_other_is_refused() {
        for mode in [0o775, 0o757, 0o1777] {
            let mut fs = FakeFs::sandbox();
            fs.nodes.insert(PathBuf::from("/mnt"), dir(0, mode, 1));
            let err = check_root_owned_ancestors(&fs, Path::new(TARGET), &[0]).unwrap_err();
            assert!(err.to_string().contains("writable"), "{mode:o}: {err}");
        }
    }

    #[test]
    fn ancestor_symlink_or_missing_is_refused() {
        let mut fs = FakeFs::sandbox();
        let mut l = dir(0, 0o777, 1);
        l.is_dir = false;
        l.is_symlink = true;
        fs.nodes.insert(PathBuf::from("/mnt/aorg-sandbox"), l);
        assert!(check_root_owned_ancestors(&fs, Path::new(TARGET), &[0]).is_err());

        let mut fs = FakeFs::sandbox();
        fs.nodes.remove(Path::new("/mnt/aorg-sandbox"));
        assert!(check_root_owned_ancestors(&fs, Path::new(TARGET), &[0]).is_err());
    }

    #[test]
    fn target_itself_is_not_an_ancestor() {
        // Before the clone the mount point does not exist; that is fine.
        let mut fs = FakeFs::sandbox();
        fs.nodes.remove(Path::new(TARGET));
        check_root_owned_ancestors(&fs, Path::new(TARGET), &[0]).unwrap();
    }

    #[test]
    fn dotted_or_relative_paths_are_refused() {
        let fs = FakeFs::sandbox();
        for p in [
            "mnt/x",
            "/mnt/../etc",
            "/mnt/./x",
            "//mnt/x",
            "/mnt/x/",
            "/",
        ] {
            assert!(
                check_root_owned_ancestors(&fs, Path::new(p), &[0]).is_err(),
                "{p}"
            );
        }
    }

    #[test]
    fn mount_of_expected_dataset_passes() {
        let fs = FakeFs::sandbox();
        assert_eq!(
            check_mount_of(&fs, Path::new(TARGET), "bigdata/rehearsal-sandbox-data").unwrap(),
            57
        );
    }

    #[test]
    fn mount_checks_refuse_the_wrong_target() {
        let ds = "bigdata/rehearsal-sandbox-data";

        // Nothing mounted there: a plain directory on the parent filesystem.
        let mut fs = FakeFs::sandbox();
        fs.mountinfo = fs.mountinfo.lines().next().unwrap().to_string();
        fs.nodes.insert(PathBuf::from(TARGET), dir(1000, 0o755, 1));
        assert!(check_mount_of(&fs, Path::new(TARGET), ds).is_err());

        // A different dataset mounted there.
        let mut fs = FakeFs::sandbox();
        fs.mountinfo = fs.mountinfo.replace(ds, "bigdata/BD/bigdata/books");
        let err = check_mount_of(&fs, Path::new(TARGET), ds).unwrap_err();
        assert!(err.to_string().contains("expected zfs"), "{err}");

        // Not zfs (e.g. a tmpfs or bind mount over the path).
        let mut fs = FakeFs::sandbox();
        fs.mountinfo = fs
            .mountinfo
            .replace(" - zfs bigdata/rehearsal", " - tmpfs bigdata/rehearsal");
        assert!(check_mount_of(&fs, Path::new(TARGET), ds).is_err());

        // Mounted, but the directory at the path is on another device
        // (something was placed over or under the mount).
        let mut fs = FakeFs::sandbox();
        fs.nodes.insert(PathBuf::from(TARGET), dir(1000, 0o755, 99));
        let err = check_mount_of(&fs, Path::new(TARGET), ds).unwrap_err();
        assert!(err.to_string().contains("does not match"), "{err}");

        // A later mount stacked on top wins.
        let mut fs = FakeFs::sandbox();
        fs.mountinfo
            .push_str("120 95 0:80 / /mnt/aorg-sandbox/data rw - tmpfs tmpfs rw\n");
        assert!(check_mount_of(&fs, Path::new(TARGET), ds).is_err());

        // A symlink at the path.
        let mut fs = FakeFs::sandbox();
        let mut l = dir(1000, 0o777, 57);
        l.is_symlink = true;
        l.is_dir = false;
        fs.nodes.insert(PathBuf::from(TARGET), l);
        assert!(check_mount_of(&fs, Path::new(TARGET), ds).is_err());

        // Prefix tricks on the mount point do not match.
        let mut fs = FakeFs::sandbox();
        fs.mountinfo = fs
            .mountinfo
            .replace("/mnt/aorg-sandbox/data rw", "/mnt/aorg-sandbox/data2 rw");
        assert!(check_mount_of(&fs, Path::new(TARGET), ds).is_err());
    }

    #[test]
    fn mountinfo_parsing() {
        let e = parse_mountinfo(
            "36 35 98:0 /mnt1 /mnt\\040two rw,noatime master:1 - ext3 /dev/root rw,errors=continue\n\
             garbage line\n",
        );
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].mount_point, "/mnt two");
        assert_eq!((e[0].major, e[0].minor), (98, 0));
        assert_eq!(e[0].fstype, "ext3");
        assert_eq!(e[0].source, "/dev/root");
    }

    #[test]
    fn bind_mount_of_a_subdirectory_is_refused() {
        let mut fs = FakeFs::sandbox();
        fs.mountinfo = fs.mountinfo.replace(
            "0:57 / /mnt/aorg-sandbox/data",
            "0:57 /some/subdir /mnt/aorg-sandbox/data",
        );
        let err =
            check_mount_of(&fs, Path::new(TARGET), "bigdata/rehearsal-sandbox-data").unwrap_err();
        assert!(err.to_string().contains("bind mount"), "{err}");
    }

    #[test]
    fn out_of_range_octal_escape_makes_the_line_unusable() {
        // \777 would overflow a byte; the kernel never writes it.
        let e = parse_mountinfo("1 1 0:57 / /mnt\\777x rw - zfs pool/x rw\n");
        assert!(e.is_empty(), "{e:?}");
        let e = parse_mountinfo("1 1 0:57 / /mnt\\377x rw - zfs pool/x rw\n");
        assert_eq!(e.len(), 1);
    }

    #[test]
    fn dev_split_uses_glibc_masks() {
        // Bits above the 32-bit major/minor ranges must not leak in.
        let dev: u64 = 0xffff_ffff_ffff_ffff;
        assert_eq!(dev_major_minor(dev), (0xffff_ffff, 0xffff_ffff));
    }

    #[test]
    fn dev_split_matches_linux_makedev() {
        assert_eq!(dev_major_minor(57), (0, 57));
        assert_eq!(dev_major_minor((8 << 8) | 1), (8, 1));
        // minor > 255 uses the high bits.
        let dev = (259u64 << 8) | (300 & 0xff) | ((300u64 & !0xff) << 12);
        assert_eq!(dev_major_minor(dev), (259, 300));
    }

    #[cfg(unix)]
    mod real_walk {
        use super::super::*;
        use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};

        fn me() -> (u32, u32) {
            // SAFETY: no arguments, cannot fail.
            unsafe { (libc::geteuid(), libc::getegid()) }
        }

        /// A scratch directory whose ancestors are not group/other-writable.
        /// The system temp dir does not qualify on Linux (`/tmp` is 1777),
        /// so use the crate's target directory instead.
        fn canon_tmp() -> (tempfile::TempDir, PathBuf) {
            let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/fs-guard-tests");
            std::fs::create_dir_all(&base).unwrap();
            std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o755)).unwrap();
            let d = tempfile::tempdir_in(&base).unwrap();
            std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
            let c = d.path().canonicalize().unwrap();
            (d, c)
        }

        #[test]
        fn walk_never_follows_symlinks_out_of_the_tree() {
            let (uid, gid) = me();
            let (_d, root) = canon_tmp();
            let tree = root.join("tree");
            let outside = root.join("outside");
            std::fs::create_dir_all(tree.join("a/b")).unwrap();
            std::fs::create_dir(&outside).unwrap();
            std::fs::write(tree.join("a/b/f"), "x").unwrap();
            std::fs::write(outside.join("secret"), "x").unwrap();
            symlink(&outside, tree.join("link-dir")).unwrap();
            symlink(outside.join("secret"), tree.join("a/link-file")).unwrap();

            // Chown to ourselves (allowed without root) and count entries.
            let dev = std::fs::metadata(&tree).unwrap().dev();
            let top = std::fs::File::open(&tree).unwrap();
            let stats = chown_tree_fd(top, dev, uid, gid).unwrap();
            // tree, a, b, f, link-dir (link itself), link-file (link itself)
            assert_eq!(stats.changed, 6, "{stats:?}");
            assert_eq!(stats.skipped_other_dev, 0);
        }

        #[test]
        fn walk_covers_wide_trees() {
            let (uid, gid) = me();
            let (_d, root) = canon_tmp();
            let tree = root.join("wide");
            for i in 0..300 {
                std::fs::create_dir_all(tree.join(format!("d{i}/inner"))).unwrap();
            }
            let dev = std::fs::metadata(&tree).unwrap().dev();
            let top = std::fs::File::open(&tree).unwrap();
            let stats = chown_tree_fd(top, dev, uid, gid).unwrap();
            assert_eq!(stats.changed, 1 + 300 * 2);
        }

        #[test]
        fn walk_refuses_wrong_device() {
            let (uid, gid) = me();
            let (_d, root) = canon_tmp();
            let dev = std::fs::metadata(&root).unwrap().dev();
            let top = std::fs::File::open(&root).unwrap();
            assert!(chown_tree_fd(top, dev.wrapping_add(1), uid, gid).is_err());
        }

        #[test]
        fn open_beneath_root_refuses_symlinked_component() {
            let (uid, _) = me();
            let (_d, root) = canon_tmp();
            std::fs::create_dir(root.join("real")).unwrap();
            symlink(root.join("real"), root.join("alias")).unwrap();
            let trusted = [0, uid];
            assert!(open_beneath_root(&root.join("real"), &trusted).is_ok());
            assert!(open_beneath_root(&root.join("alias"), &trusted).is_err());
        }

        #[test]
        fn open_beneath_root_refuses_writable_ancestor() {
            let (uid, _) = me();
            let (_d, root) = canon_tmp();
            std::fs::create_dir_all(root.join("w/target")).unwrap();
            std::fs::set_permissions(root.join("w"), std::fs::Permissions::from_mode(0o777))
                .unwrap();
            let err = open_beneath_root(&root.join("w/target"), &[0, uid]).unwrap_err();
            assert!(err.to_string().contains("writable"), "{err}");
        }

        #[test]
        fn open_beneath_root_refuses_non_root_ancestor_in_production_mode() {
            let (uid, _) = me();
            if uid == 0 {
                return;
            }
            let (_d, root) = canon_tmp();
            std::fs::create_dir(root.join("t")).unwrap();
            assert!(open_beneath_root(&root.join("t"), &[0]).is_err());
        }
    }
}
