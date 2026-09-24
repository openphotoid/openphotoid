#!/usr/bin/env python3
"""Link text that says what the link is.

A search engine reads the anchor text to decide what the destination is about,
so `[openphotoid.com](https://openphotoid.com/)` tells it nothing — the domain
is already in the href. `[openphotoid.com — free passport and ID photo maker]`
tells it what the page is (APP-134).

Flags two kinds of anchor: a bare domain or URL, and the handful of phrases
that name no subject at all ("here", "this link", "click here").

    python3 scripts/check-link-text.py [file.md ...]      # default: the READMEs
"""

import re
import sys
from urllib.parse import urlparse

DEFAULT = ["README.md", "apps/webapp/README.md"]

# `[text](target)`, with the text not itself containing a bracket.
LINK = re.compile(r"\[([^\]\[]+)\]\(([^)]+)\)")
EMPTY = {"here", "click here", "this", "this link", "link", "read more", "more"}


def host(value: str) -> str:
    """The hostname in `value`, or "" — `www.` and any trailing slash dropped."""
    parsed = urlparse(value if "//" in value else "//" + value)
    return re.sub(r"^www\.", "", parsed.netloc.lower())


def repeats_the_href(anchor: str, target: str) -> bool:
    """Anchor text that is the destination's own hostname and nothing else.

    Compared against the target rather than matched as a pattern: a filename
    like `PLAN.md` or `THIRD-PARTY-LICENSES.html` looks exactly like a domain
    to a regex, and a check that flags those is a check nobody runs twice.
    """
    parsed = urlparse(target)
    # A relative link has no scheme, and `PLAN.md` parses as a hostname if you
    # let it: only an absolute http(s) target can have its address repeated.
    if parsed.scheme not in ("http", "https") or not parsed.netloc:
        return False
    site = re.sub(r"^www\.", "", parsed.netloc.lower())
    return host(anchor) == site and "/" not in anchor.strip("/")


def check(path: str) -> list[str]:
    bad = []
    try:
        text = open(path, encoding="utf-8").read()
    except FileNotFoundError:
        return [f"{path}: not found"]
    # Anchor text may wrap across lines; join them before matching, and count
    # lines to report where it started.
    for match in LINK.finditer(text):
        anchor = " ".join(match.group(1).split())
        line = text.count("\n", 0, match.start()) + 1
        if repeats_the_href(anchor, match.group(2)):
            bad.append(f"{path}:{line}: [{anchor}] just repeats the address — say what the page is")
        elif anchor.lower() in EMPTY:
            bad.append(f"{path}:{line}: [{anchor}] names no subject")
    return bad


def main() -> int:
    paths = sys.argv[1:] or DEFAULT
    bad = [problem for path in paths for problem in check(path)]
    for problem in bad:
        print(f"::error::{problem}" if len(sys.argv) > 0 else problem)
    if bad:
        return 1
    print(f"link text: {len(paths)} file(s), every anchor says what it points at")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
