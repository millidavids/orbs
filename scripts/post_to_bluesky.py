#!/usr/bin/env python3
"""Post a release announcement to Bluesky.

Bluesky's API is open and free — an app password plus two HTTP calls, no
approval or keys. `release.yml` runs this once the tag and GitHub Release exist.

The post body comes from the `### Description` section of the version's
changelog block. That section is the release's one-line public hook, and the
same block is what the Discord embed and the GitHub Release body are built from.

Adapted from court_wizard's copy. **The Steam link is gone**, not renamed:
there is no Steam page for this game yet, and a post advertising a store link
that 404s is worse than one with three links. When there is one, add it to
`LINKS` — the facet arithmetic below is written so a link is one tuple, not a
special case.

Every post names the game in its title, so a reader who has never heard of it
can tell what they are looking at. That is not decoration: these go out to a
timeline, not to a channel someone subscribed to for this game.

Two things about the AT Protocol that are easy to get wrong, and are the reason
this is Python rather than another `curl` step:

  * A URL in the text is NOT a link. Clickable links require an explicit
    `facet`. Because a facet can attach a URL to *any* span, the links ride on
    short anchor words ("Repo", "Studio") instead of raw URLs — which matters a
    lot against a 300-character limit.
  * Facet offsets are *byte* offsets, not character offsets. The changelog is
    full of em-dashes (3 bytes each), so naive character indexing silently
    points a link at the wrong span. Offsets here are recorded while building
    the string rather than searched for afterwards, so an anchor word that also
    occurs in the description text cannot be matched by mistake.

Usage:
    post_to_bluesky.py --version 0.1.24 [--changelog docs/CHANGELOG.md]
                       [--dry-run]

Environment:
    BLUESKY_USERNAME      handle, e.g. orbs.bsky.social
    BLUESKY_APP_PASSWORD  app password from Bluesky settings (not the account password)
"""

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone

PDS = "https://bsky.social"
# Bluesky counts graphemes, not bytes. 300 is the hard limit.
MAX_GRAPHEMES = 300

GAME_NAME = "O.R.B.S."
GAME_SITE = "https://orbs.blackhearthgames.com"
STUDIO_SITE = "https://blackhearthgames.com"
REPO_SITE = "https://github.com/millidavids/orbs"
LINK_SEPARATOR = " · "

# Anchor word -> URL, in post order. Every link costs its label plus a separator
# out of the 300, which is why these are one word each.
#
# **Source is here and not in court_wizard's set** because this is a dev log for
# an unreleased game: the repository is the thing there is to look at, and there
# is no store page to send anyone to. Add Steam first when there is one, and
# consider dropping Source then — four anchors plus separators is ~35 characters
# of a 300-character budget.
LINKS = [
    ("Website", GAME_SITE),
    ("Studio", STUDIO_SITE),
    ("Source", REPO_SITE),
]


def fail(message: str):
    print(f"::error::{message}", file=sys.stderr)
    sys.exit(1)


def version_block(path: str, version: str) -> list[str]:
    """Returns the lines of the `## [v<version>]` block.

    Selects by version rather than taking the top block: a manual dispatch can
    target an older release while the changelog on main has already moved on,
    and announcing the top block there would ship the wrong notes.
    """
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError as err:
        fail(f"cannot read {path}: {err}")

    header = f"## [v{version}]"
    block: list[str] = []
    inside = False
    for line in lines:
        if line.startswith(header):
            inside = True
            continue
        if inside and line.startswith("## ["):
            break
        if inside:
            block.append(line)
    if not block:
        fail(f"no changelog block found for '{header}' in {path}")
    return block


def description(block: list[str]) -> str:
    """Pulls the prose under `### Description`.

    Falls back to the first bullet's bolded lead-in so an older release, or one
    where the section was forgotten, still produces a sensible post rather than
    failing the release after it has already been tagged and sent to Discord.
    """
    text: list[str] = []
    inside = False
    for line in block:
        if line.strip().startswith("### "):
            inside = line.strip().lower() == "### description"
            continue
        if inside and line.strip():
            text.append(line.strip())
    if text:
        return " ".join(text)

    for line in block:
        if line.startswith("- "):
            bullet = line[2:].strip()
            match = re.match(r"\*\*(.+?)\*\*\s*(?:—|-|–)?\s*(.*)", bullet, re.DOTALL)
            if match:
                lead, rest = match.group(1).strip(), match.group(2).strip()
                return f"{lead} — {rest}" if rest else lead
            return bullet.replace("**", "").strip()

    fail("changelog block has no ### Description and no bullets to fall back on")


def compose(version: str, body: str, links: list[tuple[str, str]]) -> tuple[str, list[dict]]:
    """Builds the post text and its facets, recording byte offsets as it goes."""
    title = f"{GAME_NAME} v{version}"
    footer_len = len(LINK_SEPARATOR.join(label for label, _ in links))
    budget = MAX_GRAPHEMES - len(title) - footer_len - 4  # 4 = two blank-line joins

    if len(body) > budget:
        body = body[: max(budget - 1, 0)].rsplit(" ", 1)[0].rstrip(" ,;:—–-") + "…"

    text = f"{title}\n\n{body}\n\n"
    facets: list[dict] = []
    for index, (label, url) in enumerate(links):
        if index:
            text += LINK_SEPARATOR
        start = len(text.encode("utf-8"))
        text += label
        facets.append(
            {
                "index": {"byteStart": start, "byteEnd": len(text.encode("utf-8"))},
                "features": [{"$type": "app.bsky.richtext.facet#link", "uri": url}],
            }
        )
    return text, facets


def request(url: str, payload: dict, token: str | None = None) -> dict:
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(
        url, data=json.dumps(payload).encode("utf-8"), headers=headers, method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as err:
        fail(f"{url} returned {err.code}: {err.read().decode('utf-8', 'replace')}")
    except urllib.error.URLError as err:
        fail(f"{url} unreachable: {err.reason}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--changelog", default="docs/CHANGELOG.md")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the post and exit without contacting Bluesky",
    )
    args = parser.parse_args()

    version = args.version.lstrip("v")
    body = description(version_block(args.changelog, version))
    text, facets = compose(version, body, LINKS)

    if len(text) > MAX_GRAPHEMES:
        fail(f"post is {len(text)} graphemes, over the {MAX_GRAPHEMES} limit")

    print(f"--- post ({len(text)}/{MAX_GRAPHEMES} graphemes) ---\n{text}\n---")
    for facet, (label, url) in zip(facets, LINKS):
        i = facet["index"]
        check = text.encode("utf-8")[i["byteStart"] : i["byteEnd"]].decode("utf-8")
        print(f"  facet {check!r} -> {url}" + ("" if check == label else "  MISMATCH"))

    if args.dry_run:
        return

    handle = os.environ.get("BLUESKY_USERNAME", "").strip()
    password = os.environ.get("BLUESKY_APP_PASSWORD", "").strip()
    if not handle or not password:
        fail("BLUESKY_USERNAME and BLUESKY_APP_PASSWORD must both be set")

    session = request(
        f"{PDS}/xrpc/com.atproto.server.createSession",
        {"identifier": handle, "password": password},
    )
    created_at = (
        datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    )
    result = request(
        f"{PDS}/xrpc/com.atproto.repo.createRecord",
        {
            "repo": session["did"],
            "collection": "app.bsky.feed.post",
            "record": {
                "$type": "app.bsky.feed.post",
                "text": text,
                "createdAt": created_at,
                "facets": facets,
            },
        },
        token=session["accessJwt"],
    )
    print(f"posted: {result.get('uri', '(no uri returned)')}")


if __name__ == "__main__":
    main()
