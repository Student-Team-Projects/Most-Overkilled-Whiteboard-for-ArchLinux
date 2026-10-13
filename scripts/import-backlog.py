#!/usr/bin/env python3
"""One-off (T02): turn docs/backlog.md tables into GitHub issues on a Project board.

Used once for T02 (2026-10-13); docs/backlog.md is gone since. Kept for the official-repo move.
Usage: scripts/import-backlog.py <owner/repo> <project-owner> <project-number> [--dry-run]
Creates labels and milestones, one issue per row (ID kept in the title), native blocked-by links,
and adds every issue to the project. Idempotent per title: rows whose issue exists are skipped.
"""
import json
import re
import subprocess
import sys

BACKLOG = "docs/backlog.md"
MILESTONES = {  # name -> due date (docs/roadmap.md)
    "M0": "2026-10-26", "M1": "2026-11-16", "M2": "2026-11-30",
    "M3": "2026-12-21", "M4": "2027-01-11", "M5": "2027-01-25",
}
SECTION_MILESTONE = {  # section heading prefix -> milestone
    "Grill — product": "M0", "Grill — sync": "M0", "Grill — network": "M1",
    "Grill — performance": "M3", "Grill — sim": "M0",
    "Tasks — M0": "M0", "Tasks — M1": "M1", "Tasks — M3": "M3",
}
ROW = re.compile(r"^\| ([GT]\d\d) \| (.+?) \| (.+?) \| (.+?) \|$")


def gh(*args, dry=False):
    if dry:
        print("DRY gh", " ".join(args))
        return ""
    return subprocess.run(["gh", *args], check=True, capture_output=True, text=True).stdout


def rest_id(repo, number):
    """Numeric REST id of an issue (the dependencies API wants it, not the number)."""
    return json.loads(gh("api", f"repos/{repo}/issues/{number}"))["id"]


def parse():
    rows, milestone = [], None
    for line in open(BACKLOG, encoding="utf-8"):
        if line.startswith("## "):
            head = line[3:].strip()
            milestone = next((m for p, m in SECTION_MILESTONE.items() if head.startswith(p)), None)
        elif (m := ROW.match(line.strip())) and milestone:
            rid, what, area, after = m.groups()
            rows.append({
                "id": rid, "what": what, "milestone": milestone,
                "type": "grill" if rid.startswith("G") else "task",
                "areas": [a.strip() for a in area.split(",")],
                "after": re.findall(r"[GT]\d\d", after),
                "after_text": after,
            })
    return rows


def main():
    if len(sys.argv) < 4:
        sys.exit(__doc__)
    repo, powner, pnum = sys.argv[1:4]
    dry = "--dry-run" in sys.argv
    rows = parse()
    print(f"{len(rows)} items parsed")

    labels = {"grill", "task"} | {a for r in rows for a in r["areas"]}
    for label in sorted(labels):
        gh("label", "create", label, "--repo", repo, "--force", dry=dry)
    existing_ms = json.loads(gh("api", f"repos/{repo}/milestones?state=all") or "[]")
    have = {m["title"] for m in existing_ms}
    for name, due in MILESTONES.items():
        if name not in have:
            gh("api", f"repos/{repo}/milestones", "-f", f"title={name}",
               "-f", f"due_on={due}T23:59:59Z", dry=dry)

    issues = json.loads(gh("issue", "list", "--repo", repo, "--state", "all", "--limit", "500",
                           "--json", "number,title") or "[]")
    by_id = {i["title"].split(" ")[0]: i for i in issues}
    for r in rows:
        if r["id"] in by_id:
            continue
        body = (f"Imported from `docs/backlog.md`.\n\nBlocked by: {r['after_text']}\n\n"
                + ("Closes with an ADR or a contract change.\n" if r["type"] == "grill" else ""))
        args = ["issue", "create", "--repo", repo, "--title", f"{r['id']} {r['what']}",
                "--body", body, "--milestone", r["milestone"], "--label", r["type"]]
        for a in r["areas"]:
            args += ["--label", a]
        url = gh(*args, dry=dry).strip()
        if not dry:
            num = int(url.rsplit("/", 1)[1])
            by_id[r["id"]] = {"number": num}
            gh("project", "item-add", pnum, "--owner", powner, "--url", url)

    for r in rows:  # native blocked-by; body line stays as the fallback
        for dep in r["after"]:
            if dry or dep not in by_id:
                continue
            try:
                gh("api", "-X", "POST",
                   f"repos/{repo}/issues/{by_id[r['id']]['number']}/dependencies/blocked_by",
                   "-F", f"issue_id={rest_id(repo, by_id[dep]['number'])}")
            except subprocess.CalledProcessError as e:
                print(f"blocked-by {r['id']} <- {dep} failed: {e.stderr.strip()}")
    print("done")


if __name__ == "__main__":
    main()
