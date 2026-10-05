// file: src/security/fs_guard.rs
// version: 1.0.0
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
    pub mount_point: String,
    pub fstype: String,
    pub source: String,
}

/// Undo mountinfo's octal escapes (`\040` for space and friends).
fn unescape_mountinfo(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\'
            && i + 3 < b.len()
            && b[i + 1..i + 4].iter().all(|c| (b'0'..=b'7').contains(c))
        {
            let v = (b[i + 1] - b'0') * 64 + (b[i + 2] - b'0') * 8 + (b[i + 3] - b'0');
            out.push(v);
            i += 4;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
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
        out.push(MountEntry {
            major,
            minor,
            mount_point: unescape_mountinfo(fields[4]),
            fstype: fields[sep + 1].to_string(),
            source: unescape_mountinfo(fields[sep + 2]),
        });
    }
    out
}

/// Linux `major()`/`minor()` for a 64-bit `dev_t`.
pub fn dev_major_minor(dev: u64) -> (u64, u64) {
    let major = ((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff);
    let minor = (dev & 0xff) | ((dev >> 12) & !0xff);
    (major, minor)
}

/// Check that `path` is currently the mount point of ZFS dataset `dataset`,
/// and return the device id of that mount.
///
/// The topmost mount at `path` (the last matching mountinfo line) must be
/// `zfs` with source `dataset`, and its `major:minor` must equal the `st_dev`
/// of `path` opened `O_NOFOLLOW|O_DIRECTORY`. A directory that merely sits
/// where the mount should be, a different dataset mounted there, or a
/// symlink at the path all fail.
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
    let want = path.to_string_lossy();
    let entry = parse_mountinfo(&text)
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
    if dev_major_minor(node.dev) != (entry.major, entry.minor) {
        return Err(denied(format!(
            "{} device {:?} does not match the {} mount {}:{}",
            path.display(),
            dev_major_minor(node.dev),
            dataset,
            entry.major,
            entry.minor
        )));
    }
    Ok(node.dev)
}

/// Result of a `chown-tree` walk.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChownStats {
    /// Entries whose ownership was set (including the top directory).
    pub changed: u64,
    /// Entries skipped because they are on another filesystem (nested mounts).
    pub skipped_other_dev: u64,
}

#[cfg(unix)]
mod walk {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::fs::File;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::{AsRawFd, FromRawFd};
    use std::rc::Rc;

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

    fn fstat(f: &File) -> io::Result<libc::stat> {
        // SAFETY: st is fully written by a successful fstat.
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(f.as_raw_fd(), &mut st) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(st)
    }

    fn fstatat_nofollow(dirfd: libc::c_int, name: &CStr) -> io::Result<libc::stat> {
        // SAFETY: as above; AT_SYMLINK_NOFOLLOW stats a symlink itself.
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstatat(dirfd, name.as_ptr(), &mut st, libc::AT_SYMLINK_NOFOLLOW) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(st)
    }

    fn is_dir(st: &libc::stat) -> bool {
        (st.st_mode as u32 & libc::S_IFMT as u32) == libc::S_IFDIR as u32
    }

    /// Open `path` one component at a time from `/`, never following a
    /// symlink, and check each directory opened above the target through its
    /// descriptor (owner in `trusted_uids`, not group/other-writable). This
    /// repeats the ancestor precondition on the descriptors actually used,
    /// so swapping a component after the precondition ran does not help.
    pub fn open_beneath_root(path: &Path, trusted_uids: &[u32]) -> io::Result<File> {
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
            let st = fstat(&cur)?;
            if !trusted_uids.contains(&st.st_uid) || (st.st_mode as u32) & 0o022 != 0 {
                return Err(denied(format!(
                    "ancestor {} is not root-owned or is writable by group or other",
                    shown.display()
                )));
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
        loop {
            // SAFETY: dp is a valid DIR*; the returned entry is valid until
            // the next readdir/closedir, and we copy the name out at once.
            let ent = unsafe { libc::readdir(dp) };
            if ent.is_null() {
                break;
            }
            let name = unsafe { CStr::from_ptr((*ent).d_name.as_ptr()) };
            let b = name.to_bytes();
            if b != b"." && b != b".." {
                names.push(name.to_owned());
            }
        }
        unsafe { libc::closedir(dp) };
        Ok(names)
    }

    /// Change every non-directory entry of `dir` (same device only) and queue
    /// its subdirectories.
    fn list_into(
        dir: &Rc<File>,
        dev: u64,
        uid: u32,
        gid: u32,
        stats: &mut ChownStats,
        pending: &mut Vec<(Rc<File>, CString)>,
    ) -> io::Result<()> {
        for name in read_names(dir)? {
            let st = match fstatat_nofollow(dir.as_raw_fd(), &name) {
                Ok(st) => st,
                Err(e) if e.raw_os_error() == Some(libc::ENOENT) => continue,
                Err(e) => return Err(e),
            };
            if st.st_dev as u64 != dev {
                stats.skipped_other_dev += 1;
                continue;
            }
            if is_dir(&st) {
                pending.push((Rc::clone(dir), name));
                continue;
            }
            // Files, symlinks (the link itself), fifos, sockets, device
            // nodes. AT_SYMLINK_NOFOLLOW never follows.
            // SAFETY: dir's fd is open for the call; name is NUL-terminated.
            let rc = unsafe {
                libc::fchownat(
                    dir.as_raw_fd(),
                    name.as_ptr(),
                    uid,
                    gid,
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if rc != 0 {
                let e = io::Error::last_os_error();
                if e.raw_os_error() == Some(libc::ENOENT) {
                    continue;
                }
                return Err(e);
            }
            stats.changed += 1;
        }
        Ok(())
    }

    /// Set `uid:gid` on `top` and everything beneath it on the same
    /// filesystem (`dev`), never following symlinks.
    pub fn chown_tree_fd(top: File, dev: u64, uid: u32, gid: u32) -> io::Result<ChownStats> {
        let mut stats = ChownStats::default();
        let st = fstat(&top)?;
        if st.st_dev as u64 != dev || !is_dir(&st) {
            return Err(denied("top of tree is not the expected directory".into()));
        }
        // SAFETY: fd is open; fchown on a descriptor cannot be redirected.
        if unsafe { libc::fchown(top.as_raw_fd(), uid, gid) } != 0 {
            return Err(io::Error::last_os_error());
        }
        stats.changed += 1;

        // Depth-first. Pending subdirectories are kept as (parent, name) and
        // opened only when popped, so the number of open descriptors grows
        // with depth, not with the number of directories in a wide tree.
        let top = Rc::new(top);
        let mut pending: Vec<(Rc<File>, CString)> = Vec::new();
        list_into(&top, dev, uid, gid, &mut stats, &mut pending)?;
        while let Some((parent, name)) = pending.pop() {
            let sub = match openat_dir(parent.as_raw_fd(), &name) {
                Ok(f) => f,
                // Removed, or replaced by a symlink or a non-directory since
                // it was listed (the sandbox user owns the tree). O_NOFOLLOW
                // makes a swapped-in symlink fail here rather than be followed.
                Err(e)
                    if matches!(
                        e.raw_os_error(),
                        Some(libc::ENOENT) | Some(libc::ELOOP) | Some(libc::ENOTDIR)
                    ) =>
                {
                    continue
                }
                Err(e) => return Err(e),
            };
            drop(parent);
            let sst = fstat(&sub)?;
            if sst.st_dev as u64 != dev {
                stats.skipped_other_dev += 1;
                continue;
            }
            // SAFETY: fd is open; fchown on a descriptor cannot be redirected.
            if unsafe { libc::fchown(sub.as_raw_fd(), uid, gid) } != 0 {
                return Err(io::Error::last_os_error());
            }
            stats.changed += 1;
            let sub = Rc::new(sub);
            list_into(&sub, dev, uid, gid, &mut stats, &mut pending)?;
        }
        Ok(stats)
    }
}

#[cfg(unix)]
pub use walk::{chown_tree_fd, open_beneath_root};

/// Run a whole `chown-tree`: open the target without following symlinks
/// (re-checking ancestors on the descriptors), confirm it is still the
/// device the mount check found, then walk it.
#[cfg(unix)]
pub fn chown_tree(
    path: &Path,
    expected_dev: u64,
    uid: u32,
    gid: u32,
    trusted_uids: &[u32],
) -> io::Result<ChownStats> {
    let top = open_beneath_root(path, trusted_uids)?;
    chown_tree_fd(top, expected_dev, uid, gid)
}

#[cfg(not(unix))]
pub fn chown_tree(
    _path: &Path,
    _expected_dev: u64,
    _uid: u32,
    _gid: u32,
    _trusted_uids: &[u32],
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

        fn canon_tmp() -> (tempfile::TempDir, PathBuf) {
            let d = tempfile::tempdir().unwrap();
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
