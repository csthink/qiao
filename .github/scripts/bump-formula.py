#!/usr/bin/env python3
"""Surgically bump the llmkeys Homebrew formula in place.

Only rewrites the volatile fields the release produces:
  - the version embedded in both release-download URLs
  - the sha256 inside the on_arm block
  - the sha256 inside the on_intel block

Everything else (desc, homepage, license, caveats, test) is left untouched —
the tap repo remains the owner of those, the release pipeline only owns the
download coordinates. Fails loudly if any expected field is missing so a
malformed formula never gets pushed.

Usage: bump-formula.py <path-to-formula.rb>
Reads VERSION / ARM_SHA / INTEL_SHA from the environment.
"""
import os
import re
import sys
import pathlib


def require_env(name: str) -> str:
    val = os.environ.get(name)
    if not val:
        sys.exit(f"error: ${name} is empty or unset")
    return val


def sub_once(pattern: str, repl, text: str, what: str, flags=0) -> str:
    new, n = re.subn(pattern, repl, text, count=1, flags=flags)
    if n != 1:
        sys.exit(f"error: expected exactly one match for {what}, got {n}")
    return new


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit("usage: bump-formula.py <path-to-formula.rb>")

    version = require_env("VERSION")
    arm_sha = require_env("ARM_SHA")
    intel_sha = require_env("INTEL_SHA")

    for name, sha in (("ARM_SHA", arm_sha), ("INTEL_SHA", intel_sha)):
        if not re.fullmatch(r"[0-9a-f]{64}", sha):
            sys.exit(f"error: ${name} is not a 64-char hex sha256: {sha!r}")

    path = pathlib.Path(sys.argv[1])
    src = path.read_text()
    out = src

    # Bump the version inside both release-download URLs. Version charset is
    # digits/dots only — the release filename is `llmkeys-<ver>-<target>` and
    # the target itself contains hyphens, so allowing '-' here would let the
    # match bleed into the target. Safe because the update-tap job excludes
    # prerelease ('-') tags upstream.
    out = re.sub(
        r"(/releases/download/v)[0-9][0-9.]*(/llmkeys-)[0-9][0-9.]*(-)",
        rf"\g<1>{version}\g<2>{version}\g<3>",
        out,
    )

    # Replace the sha256 within each architecture block, scoped so arm/intel
    # never get crossed.
    out = sub_once(
        r"(on_arm do.*?sha256 \")[0-9a-f]{64}",
        rf"\g<1>{arm_sha}",
        out,
        "on_arm sha256",
        flags=re.S,
    )
    out = sub_once(
        r"(on_intel do.*?sha256 \")[0-9a-f]{64}",
        rf"\g<1>{intel_sha}",
        out,
        "on_intel sha256",
        flags=re.S,
    )

    if out == src:
        print(f"formula already at v{version}; nothing to change")
        return

    path.write_text(out)
    print(f"formula bumped to v{version}")
    print(f"  arm   sha256 = {arm_sha}")
    print(f"  intel sha256 = {intel_sha}")


if __name__ == "__main__":
    main()
