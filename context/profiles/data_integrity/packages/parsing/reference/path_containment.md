# Path Containment and Symlink Safety

A tool that reads or writes inside a root directory must guarantee that no input path, symlink, or race takes an operation outside it. Linux and macOS only. Crate and std facts were verified against std 1.99.0, cap-std 4.0.3, and rustix 1.1.5 (2026-10-04).

## Canonical Root

- Canonicalize the root once at startup with `fs::canonicalize`. It resolves symlinks (`realpath` on Unix) and fails if the path does not exist, so create the root before canonicalizing it. A missing root is a startup error, not something to recover from later.
- Compare canonical forms only, never a raw input path against the canonical root.
- A configured root may resolve through a symlinked alias, such as `/var` on macOS. Always compare canonicalized forms; never hard-code an alias expansion.

## Containment Checks

- Test containment with `Path::starts_with`, which "only considers whole path components to match". Never use a string prefix: `"/repo-evil".starts_with("/repo")` is true for strings and false for paths.
- The candidate must be canonical too. For an existing target, canonicalize it and check it against the canonical root. For a target that does not exist yet (a write), canonicalize its parent directory, check the parent, and then join a validated final name.

## Untrusted Relative Inputs

Validate every untrusted relative path component by component before joining it to the root:

- Reject any `ParentDir` (`..`), `RootDir`, or `Prefix` component, and any absolute input. `root.join(abs)` replaces the root entirely.
- Do not rely on `components()` to sanitize. It drops repeated separators, interior `.`, and trailing separators, but "no other normalization takes place": `a/b/../c` stays distinct from `a/c`, because `b` may be a symlink. Lexical normalization is never a substitute for rejection.
- Reject empty inputs and names containing NUL.

## Symlink Policy per File Class

Inspect without following by using `fs::symlink_metadata` (`lstat`), and declare a policy for each file class rather than inheriting the OS default:

| File class | Policy |
|---|---|
| Authored inputs (read) | Follow only if the canonicalized target stays inside the root; otherwise refuse |
| Writes, creates, renames | Never follow; refuse if the final component is a symlink |
| Deletes | Never follow; remove the link itself, never its target |
| Enumeration | Report symlinks as symlinks; do not descend through them |

## TOCTOU: Check, Then Use, Is Racy

A check by path followed by an open by path leaves a window in which a component can be swapped for a symlink. Close the window by binding the check to the handle you actually use:

- Prefer directory-handle-relative operations. cap-std's `Dir` (from `Dir::open_ambient_dir(path, ambient_authority)`) opens files "via paths relative to a `Dir`". rustix exposes `openat(dirfd, path, oflags, …)` with `OFlags::NOFOLLOW`.
- With std alone, pass `libc::O_NOFOLLOW` through `OpenOptionsExt::custom_flags`. Custom flags can only set flags, never clear them. `O_NOFOLLOW` covers only the final component; intermediate components still need handle-relative traversal.
- Recheck after opening: `fstat` the handle (`File::metadata`) and confirm the file type and, where it matters, the device and inode against what was validated.
- Create with `OpenOptions::create_new(true)`, so an existing file or planted symlink fails the open instead of being truncated or followed.
- Recursive deletion is the classic case. std's `fs::remove_dir_all` was vulnerable to a symlink race (CVE-2022-21658, Rust 1.0.0 through 1.58.0, fixed in 1.58.1) and now uses `openat`/`unlinkat`/`lstat` on Unix. A hand-rolled recursive walk must meet the same bar.

## Platform Differences

- Treat case sensitivity as a property of the volume, not the OS. On macOS, case-sensitive APFS is a separate, opt-in format variant, so both behaviors occur in practice. Detect it at runtime by probing the actual root: create a probe file under a unique name, then stat a case-flipped spelling of that name. Never infer it from `cfg!(target_os)`.
- On a case-insensitive root, names differing only in case collide; duplicate detection must use the probed behavior.
- No Windows code paths: no drive prefixes, UNC handling, or `\` separators.

## Diagnostics

An escape attempt is a typed error variant that names the input path (and the resolved path, when one exists): `PathError::Escapes { input, resolved }`, `PathError::SymlinkRefused { input }`, `PathError::InvalidComponent { input, component }`. Never silently clamp an escaping path to the root, strip `..`, or skip the file. Every refusal is visible to the caller.

## Sketch

This is a sketch, not a drop-in implementation:

```rust
fn resolve(root: &Path /* canonical */, input: &Path) -> Result<PathBuf, PathError> {
    for c in input.components() {
        match c {
            Component::Normal(_) | Component::CurDir => {}
            other => return Err(PathError::InvalidComponent { input: input.into(), component: format!("{other:?}") }),
        }
    }
    let joined = root.join(input);
    let resolved = fs::canonicalize(&joined).map_err(|e| PathError::Io { input: input.into(), source: e })?;
    if !resolved.starts_with(root) {                       // component-wise
        return Err(PathError::Escapes { input: input.into(), resolved });
    }
    Ok(resolved)                                           // still re-validate the opened handle
}
```

## Verification

Use fixtures covering `..` at every position, absolute inputs, a string-prefix sibling of the root, symlinks to targets inside and outside the root, a symlinked intermediate directory, a dangling symlink, a pre-existing `create_new` target, and case-flipped names on both kinds of root. Assert the exact error variant and named input for each refusal.

## Related

- Typed error enums, error context, and the diagnostic/error split for refusal variants: [Error Handling](../../../../core/packages/errors/reference/error_handling.md)
- Atomic publication and untrusted-input posture for files written inside the root: [Persistence Envelopes for Zero-Copy Archives](../../persistence/reference/persistence_envelopes.md)
