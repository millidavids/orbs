#!/usr/bin/env python3
"""Prove a change touched comments and nothing else.

The comment pass's gate. Each changed `.rs`, `.toml` and `.wgsl` file is lexed
in both versions with its comments dropped, and the token streams compared.
Tokens, not lines, so rustfmt rewrapping a line whose trailing comment went is
not a difference. Kept as code: string literals, raw strings, a `#` inside a
TOML string, and every line inside a doc comment's ``` fence, since a doctest
compiles.

Nothing on `dev` is committed between boxes, so the base is a snapshot of the
working tree taken through a private index, never `HEAD`:

    base=$(scripts/only-comments.py snapshot)
    ...edit comments...
    scripts/only-comments.py check "$base"
    scripts/only-comments.py self-test     # the negative control

`check` exits 1 on a token changed, a file added or removed, or a file type it
cannot lex.
"""

import os
import subprocess
import sys
import tempfile

LEXED = (".rs", ".toml", ".wgsl")


def git(*args, env=None):
    return subprocess.run(
        ["git", *args], check=True, capture_output=True, env=env
    ).stdout


def snapshot():
    """A tree object holding the working tree as it stands, index untouched."""
    git_dir = git("rev-parse", "--git-dir").decode().strip()
    fd, index = tempfile.mkstemp(prefix="only-comments.", dir=git_dir)
    os.close(fd)
    try:
        env = dict(os.environ, GIT_INDEX_FILE=index)
        git("read-tree", "HEAD", env=env)
        git("add", "-A", env=env)
        return git("write-tree", env=env).decode().strip()
    finally:
        os.unlink(index)


# --- Rust and WGSL ----------------------------------------------------------


def rust_tokens(text):
    """`(token, line)` for everything that is not a comment."""
    out = []
    i, n, line = 0, len(text), 1

    def take(start, end, kind):
        out.append((kind + text[start:end], line_at(start)))

    starts = [0]
    for k, ch in enumerate(text):
        if ch == "\n":
            starts.append(k + 1)

    def line_at(pos):
        lo, hi = 0, len(starts) - 1
        while lo < hi:
            mid = (lo + hi + 1) // 2
            if starts[mid] <= pos:
                lo = mid
            else:
                hi = mid - 1
        return lo + 1

    in_fence = False
    while i < n:
        ch = text[i]
        if ch.isspace():
            i += 1
            continue
        if text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end == -1 else end
            body = text[i:end]
            # A doc comment's fenced block compiles as a doctest: its lines are
            # code. The fence lines themselves count too.
            doc = body.startswith("///") or body.startswith("//!")
            if doc:
                inner = body[3:].strip()
                if inner.startswith("```"):
                    in_fence = not in_fence
                    out.append(("doc" + inner, line_at(i)))
                elif in_fence:
                    out.append(("doc" + " ".join(inner.split()), line_at(i)))
            i = end
            continue
        if text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            i = j
            continue
        # Raw strings: r"..", r#".."#, br#".."#, cr".."
        j = i
        if text[j] in "bc" and j + 1 < n and text[j + 1] == "r":
            j += 1
        if text[j] == "r" and j + 1 < n and text[j + 1] in '#"':
            k = j + 1
            hashes = 0
            while k < n and text[k] == "#":
                hashes, k = hashes + 1, k + 1
            if k < n and text[k] == '"':
                close = '"' + "#" * hashes
                end = text.find(close, k + 1)
                end = n if end == -1 else end + len(close)
                take(i, end, "str:")
                i = end
                continue
        # Strings: "..", b"..", c".."
        j = i + 1 if text[i] in "bc" and i + 1 < n and text[i + 1] == '"' else i
        if text[j] == '"':
            k = j + 1
            while k < n and text[k] != '"':
                k += 2 if text[k] == "\\" else 1
            take(i, k + 1, "str:")
            i = k + 1
            continue
        # A char literal, or a lifetime or label.
        if ch == "'" or (ch == "b" and i + 1 < n and text[i + 1] == "'"):
            q = i if ch == "'" else i + 1
            if q + 1 < n and text[q + 1] == "\\":
                k = q + 2
                while k < n and text[k] != "'":
                    k += 1
                take(i, k + 1, "chr:")
                i = k + 1
                continue
            if q + 2 < n and text[q + 2] == "'":
                take(i, q + 3, "chr:")
                i = q + 3
                continue
            k = q + 1
            while k < n and (text[k].isalnum() or text[k] == "_"):
                k += 1
            take(i, k, "life:")
            i = k
            continue
        if ch.isalnum() or ch == "_":
            k = i
            while k < n and (text[k].isalnum() or text[k] == "_"):
                k += 1
            # A float's fraction, so `1.0` is one token and `x.0` stays three.
            if text[i].isdigit() and k + 1 < n and text[k] == "." and text[k + 1].isdigit():
                k += 1
                while k < n and (text[k].isalnum() or text[k] == "_"):
                    k += 1
            take(i, k, "")
            i = k
            continue
        take(i, i + 1, "")
        i += 1
    return out


# --- TOML -------------------------------------------------------------------


def toml_tokens(text):
    out = []
    for number, raw in enumerate(text.splitlines(), 1):
        out.extend((token, number) for token in toml_line(raw))
    return out


def toml_line(raw, state=None):
    # Multi-line strings are handled by `toml_tokens_ml`; this is one line.
    tokens, i, n = [], 0, len(raw)
    while i < n:
        ch = raw[i]
        if ch.isspace():
            i += 1
        elif ch == "#":
            break
        elif ch in "\"'":
            k = i + 1
            while k < n and raw[k] != ch:
                k += 2 if ch == '"' and raw[k] == "\\" else 1
            tokens.append("str:" + raw[i : k + 1])
            i = k + 1
        else:
            k = i
            while k < n and not raw[k].isspace() and raw[k] not in "#\"'":
                k += 1
            tokens.append(raw[i:k])
            i = k
    return tokens


def toml_tokens_ml(text):
    """TOML with multi-line strings kept whole, whatever `#` they hold."""
    out, i, n, line = [], 0, len(text), 1
    while i < n:
        for quote in ('"""', "'''"):
            if text.startswith(quote, i):
                end = text.find(quote, i + 3)
                end = n if end == -1 else end + 3
                out.append(("str:" + text[i:end], line))
                line += text.count("\n", i, end)
                i = end
                break
        else:
            end = text.find("\n", i)
            end = n if end == -1 else end
            chunk = text[i:end]
            # A line that opens a multi-line string part way along.
            cut = min(
                (p for p in (chunk.find('"""'), chunk.find("'''")) if p != -1),
                default=-1,
            )
            if cut != -1 and "#" not in chunk[:cut]:
                out.extend((t, line) for t in toml_line(chunk[:cut]))
                i += cut
                continue
            out.extend((t, line) for t in toml_line(chunk))
            i = end + 1
            line += 1
    return out


# --- comparing --------------------------------------------------------------


def tokens_of(path, text):
    if path.endswith((".rs", ".wgsl")):
        return rust_tokens(text)
    return toml_tokens_ml(text)


def compare(path, before, after):
    """None if only comments differ, else a sentence saying where."""
    old, new = tokens_of(path, before), tokens_of(path, after)
    for k, (a, b) in enumerate(zip(old, new)):
        if a[0] != b[0]:
            return f"line {a[1]} → {b[1]}: {a[0]!r} became {b[0]!r}"
    if len(old) != len(new):
        longer = old if len(old) > len(new) else new
        extra = longer[min(len(old), len(new))]
        side = "removed" if len(old) > len(new) else "added"
        return f"line {extra[1]}: {extra[0]!r} {side}"
    return None


def check(base):
    current = snapshot()
    changed = git("diff", "--name-status", "--no-renames", base, current).decode()
    failures = 0
    files = 0
    for row in changed.splitlines():
        status, path = row.split("\t", 1)
        files += 1
        if status != "M":
            print(f"FAIL {path}: {'added' if status == 'A' else 'removed'}")
            failures += 1
            continue
        if not path.endswith(LEXED):
            print(f"FAIL {path}: not a file this can lex")
            failures += 1
            continue
        before = git("show", f"{base}:{path}").decode()
        after = git("show", f"{current}:{path}").decode()
        why = compare(path, before, after)
        if why:
            print(f"FAIL {path}: {why}")
            failures += 1
    print(f"{files} files, {failures} with more than comments changed")
    return 1 if failures else 0


# --- the negative control ---------------------------------------------------

RUST = '''//! A module.
/// Adds.
///
/// ```
/// assert_eq!(add(1, 2), 3);
/// ```
pub fn add<'a>(a: u32, b: u32) -> u32 { // trailing
    let s = r#"not // a comment"#; /* a /* nested */ block */
    let c = '/'; let q = '\\''; let l: &'a str = "x // y";
    a + b
}
'''

TOML = '''# a comment
[spell]
body = "#!/orbs/invoke # not a comment"
lines = """
# still a string
"""
count = 3 # trailing
'''


def self_test():
    cases = [
        ("rs", RUST, RUST.replace("/// Adds.", "/// Adds two."), None),
        ("rs", RUST, RUST.replace(" // trailing", ""), None),
        ("rs", RUST, RUST.replace("/* a /* nested */ block */", "/* x */"), None),
        ("rs", RUST, RUST.replace("a + b", "a - b"), "fail"),
        ("rs", RUST, RUST.replace("not // a comment", "not a comment"), "fail"),
        ("rs", RUST, RUST.replace("x // y", "x"), "fail"),
        ("rs", RUST, RUST.replace("'/'", "'.'"), "fail"),
        ("rs", RUST, RUST.replace("&'a str", "&'b str"), "fail"),
        ("rs", RUST, RUST.replace("add(1, 2), 3", "add(1, 2), 4"), "fail"),
        ("rs", RUST, RUST.replace("{ // trailing\n    let", "{\n let"), None),
        ("toml", TOML, TOML.replace("# a comment", "# another"), None),
        ("toml", TOML, TOML.replace(" # trailing", ""), None),
        ("toml", TOML, TOML.replace("# not a comment", ""), "fail"),
        ("toml", TOML, TOML.replace("# still a string", "# changed"), "fail"),
        ("toml", TOML, TOML.replace("count = 3", "count = 4"), "fail"),
    ]
    wrong = 0
    for number, (kind, before, after, expected) in enumerate(cases, 1):
        assert before != after, f"case {number} changes nothing"
        got = compare(f"x.{kind}", before, after)
        if (got is None) != (expected is None):
            wrong += 1
            print(f"case {number}: expected {expected or 'pass'}, got {got or 'pass'}")
    print(f"{len(cases)} cases, {wrong} judged wrongly")
    return 1 if wrong else 0


def main():
    if len(sys.argv) >= 2 and sys.argv[1] == "snapshot":
        print(snapshot())
        return 0
    if len(sys.argv) == 3 and sys.argv[1] == "check":
        return check(sys.argv[2])
    if len(sys.argv) == 2 and sys.argv[1] == "self-test":
        return self_test()
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
