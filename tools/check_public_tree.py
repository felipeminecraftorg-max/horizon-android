#!/usr/bin/env python3
"""Reject local game artifacts even when somebody uses git add -f."""

import argparse
from pathlib import Path, PurePosixPath
import subprocess
import sys

PRIVATE_ROOTS = {".local", "decompiled", "minecraft", "assets", "run", "logs"}
PRIVATE_EXTENSIONS = {".jar", ".class", ".jfr", ".hprof", ".java", ".apk", ".aab", ".keystore", ".jks"}
ORIGINAL_JDK_TOOLS = {"tools/JfrSummary.java", "tests/java/JfrFixture.java"}
# Owner-approved, original NativeActivity result callback; not recovered game code.
ANDROID_RESULT_BRIDGE = "packaging/android/java/dev/horizon/android/HorizonActivity.java"
ANDROID_DOCUMENTS_BRIDGE = "packaging/android/java/dev/horizon/android/HorizonDocumentsProvider.java"
# Owner-approved (2026-11-08): the mod converter's lane 3 on Android, its runtime and the Java API
# slice Java mods link against. Original code written for Horizon (named as Mojang's, Fabric's
# and the loggers' API so a mod's calls resolve), never recovered game code. An exact list
# (LANE3_LIST, one path a line), and every listed file in the tree starts with LANE3_HEADER.
LANE3_LIST = Path(__file__).with_name("android_lane3_sources.txt")
LANE3_HEADER = "// Horizon Android, lane 3 of the mod converter: written for Horizon"


def lane3_sources():
    try:
        lines = LANE3_LIST.read_text(encoding="utf-8").splitlines()
    except OSError:
        return frozenset()
    return frozenset(line.strip() for line in lines if line.strip() and not line.startswith("#"))


LANE3 = lane3_sources()


def lane3_source(path):
    return path in LANE3


def prohibited(path):
    parsed = PurePosixPath(path)
    if parsed.name.lower() in {"local.properties", "control.json", "android-launch.json", "horizon-touch.json", "engine-data.zip", "horizon-resources.json"}:
        return True
    name = parsed.name.lower()
    if name == ".env" or (name.startswith(".env.") and name != ".env.example"):
        return True
    if any(part in {".local", ".gradle", "node_modules", "__pycache__"} for part in parsed.parts):
        return True
    # Original JDK-only tools are an exact allowlist; recovered game Java stays private.
    if path in ORIGINAL_JDK_TOOLS or path in {ANDROID_RESULT_BRIDGE, ANDROID_DOCUMENTS_BRIDGE}:
        return False
    if lane3_source(path):
        return False
    # Original Horizon mod sources may also be tracked.
    if parsed.suffix == ".java" and path.startswith(("mods/horizon/src/", "mods/horizon-lab/src/")):
        return not ("/dev/horizonlab/" in path)
    return bool(parsed.parts and parsed.parts[0] in PRIVATE_ROOTS) or parsed.suffix.lower() in PRIVATE_EXTENSIONS


def paths_in_repository(history=False, revisions=None):
    if not history:
        result = subprocess.check_output(["git", "ls-files", "-z"])
        return result.decode("utf-8").split("\0")
    # An empty push (including deletions only) must not fall back to HEAD or --all.
    if revisions is not None and not revisions:
        return []
    roots = ["--all"] if revisions is None else revisions
    # Include diffs against merge parents so merge-resolution-only artifacts are caught.
    # Git pushes the stored objects, not the history presented by local replace refs.
    result = subprocess.check_output(["git", "--no-replace-objects", "log", "--format=", "--name-only",
                                      "-z", "-m", *roots, "--"])
    return [path.lstrip("\n") for path in result.decode("utf-8").split("\0")]


def pushed_revisions(stream):
    """Read Git's pre-push ref updates and retain the non-deleted local object IDs."""
    revisions = []
    for line in stream:
        _local_ref, local_oid, _remote_ref, _remote_oid = line.split()
        if local_oid.strip("0"):
            revisions.append(local_oid)
    return revisions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--history", action="store_true", help="Inspect paths in all local Git history")
    mode.add_argument("--pre-push", action="store_true", help="Inspect history reachable from Git's ref updates on stdin")
    # CI's pull requests: only the commits a merge would add, not every ref's whole history.
    mode.add_argument("--range", metavar="BASE..HEAD", help="Inspect the commits of a revision range")
    args = parser.parse_args()
    revisions = pushed_revisions(sys.stdin) if args.pre_push else [args.range] if args.range else None
    history = args.history or args.pre_push or bool(args.range)
    rejected = sorted({path for path in paths_in_repository(history, revisions)
                       if path and prohibited(path)})
    if not history:
        # A lane 3 file in the tree must say it is Horizon's own; any other is rejected.
        for path in paths_in_repository(False):
            if path and lane3_source(path):
                try:
                    with open(path, encoding="utf-8") as source:
                        first = source.readline()
                except OSError:
                    continue
                if not first.startswith(LANE3_HEADER):
                    rejected.append(path)
        rejected.sort()
    if rejected:
        print("Private game/profiling artifacts must stay outside Git:", file=sys.stderr)
        for path in rejected:
            print(f"  {path}", file=sys.stderr)
        return 1
    print("Public tree check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
