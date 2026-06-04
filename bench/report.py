#!/usr/bin/env python3
"""Post-process criterion benchmark results into chenal-centric markdown tables.

The companion benchmark (benches/bench.rs) compares the `chenal` channel crate
against several competing channel crates. Criterion writes per-benchmark results
under ``target/criterion/<mangled-dir>/new/{benchmark.json,estimates.json}``.

This script discovers every result by globbing for ``new/benchmark.json`` files
(directory names are deliberately ignored because they are mangled and
ambiguous), reads the sibling ``estimates.json`` for the timing, and emits one
markdown table per ``(kind, op)`` scenario. Each table shows ns-per-message for
every crate plus its ratio relative to chenal (the baseline).

Timings: criterion reports nanoseconds for one iteration, and each iteration
processes ``MESSAGE_COUNT = 1024`` messages, so ns/msg = point_estimate / 1024.
``slope.point_estimate`` is preferred when present, otherwise
``mean.point_estimate``.

Usage::

    python3 report.py [--criterion-dir DIR] > RESULTS.md
"""

from __future__ import annotations

import argparse
import glob
import json
import os
from collections import defaultdict

# Each criterion iteration processes this many messages (see bench.rs).
MESSAGE_COUNT = 1024

# Recognized scenario tokens. Order-independent parsing keys off these sets so
# the parser is robust to future reordering of group_id tokens.
KINDS = {"mpsc", "mpmc", "spmc", "spsc"}
OPS = {
    "try_send",
    "try_recv",
    "poll_send",
    "poll_recv",
    "send_async",
    "recv_async",
    "send_blocking",
    "recv_blocking",
}
# Tokens belonging to the old benchmark format; their presence disqualifies a
# group_id from the current token scheme.
STALE_TOKENS_PREFIXES = ("senders=", "receivers=", "capacity=")
# Bare-word tokens that mark stale/old-format scenarios.
STALE_BARE_TOKENS = {"chenal_loop", "seq", "blocking", "async", "racy", "mpmc_racy"}


def parse_scenario(group_id):
    """Parse a group_id into (kind, op, msg_size, contended).

    Returns ``None`` for any group_id that does not match the current token
    scheme (stale/old-format directories are silently rejected).

    ``contended`` is ``True``/``False`` for send/recv ops, or ``None`` for ops
    without a contention dimension (try_/poll_).
    """
    if not group_id:
        return None

    kind = None
    op = None
    msg_size = None
    contended = None

    for tok in group_id.split("/"):
        if tok in KINDS:
            if kind is not None:
                return None
            kind = tok
        elif tok in OPS:
            if op is not None:
                return None
            op = tok
        elif tok.startswith("msg_size="):
            try:
                msg_size = int(tok[len("msg_size="):])
            except ValueError:
                return None
        elif tok.startswith("contended="):
            val = tok[len("contended="):]
            if val == "true":
                contended = True
            elif val == "false":
                contended = False
            else:
                return None
        else:
            # Any unrecognized token (stale params, typos like "contented=",
            # old bare words) disqualifies the scenario.
            return None

    if kind is None or op is None or msg_size is None:
        return None

    return (kind, op, msg_size, contended)


def point_estimate(estimates):
    """Extract the preferred point estimate (ns/iteration) from estimates.json."""
    slope = estimates.get("slope")
    if isinstance(slope, dict) and "point_estimate" in slope:
        return slope["point_estimate"]
    mean = estimates.get("mean")
    if isinstance(mean, dict) and "point_estimate" in mean:
        return mean["point_estimate"]
    return None


def collect(criterion_dir):
    """Return ``{scenario: {crate: ns_per_msg}}`` for all valid results."""
    results = defaultdict(dict)
    pattern = os.path.join(criterion_dir, "**", "new", "benchmark.json")
    for bench_path in glob.glob(pattern, recursive=True):
        try:
            with open(bench_path) as fh:
                bench = json.load(fh)
        except (OSError, ValueError):
            continue

        crate = bench.get("function_id")
        if not crate:
            # Old-format entries embed the crate in group_id and leave
            # function_id null; they are not part of the current scheme.
            continue

        scenario = parse_scenario(bench.get("group_id"))
        if scenario is None:
            continue

        est_path = os.path.join(os.path.dirname(bench_path), "estimates.json")
        try:
            with open(est_path) as fh:
                estimates = json.load(fh)
        except (OSError, ValueError):
            continue

        pe = point_estimate(estimates)
        if pe is None:
            continue

        results[scenario][crate] = pe / MESSAGE_COUNT
    return results


def fmt_ns(value):
    """Format a ns/msg value to ~3 significant figures."""
    if value == 0:
        return "0"
    if value >= 100:
        return f"{value:.0f}"
    if value >= 10:
        return f"{value:.1f}"
    if value >= 1:
        return f"{value:.2f}"
    return f"{value:.3g}"


def param_label(msg_size, contended):
    if contended is None:
        return f"msg_size={msg_size}"
    return f"msg_size={msg_size}, contended={'true' if contended else 'false'}"


def build_tables(results):
    """Group scenarios by (kind, op) and render markdown for each."""
    # (kind, op) -> { (msg_size, contended) -> {crate: ns} }
    grouped = defaultdict(lambda: defaultdict(dict))
    for (kind, op, msg_size, contended), per_crate in results.items():
        col = (msg_size, contended)
        for crate, ns in per_crate.items():
            grouped[(kind, op)][col][crate] = ns

    out = []
    for (kind, op) in sorted(grouped):
        columns = grouped[(kind, op)]

        # Collect crate set across all columns of this table.
        crates = set()
        for col in columns:
            crates.update(columns[col])

        # Drop partial columns: a column is only shown when every crate in the
        # table has a result for it. Partial columns arise from downgrading a
        # cloneable mpmc/mpsc channel into an spsc/mpsc scenario — only those
        # cloneable crates produce the contended=true variant, so that column is
        # missing the genuine single-producer crates and is not comparable.
        col_keys = sorted(
            (c for c in columns if all(crate in columns[c] for crate in crates)),
            key=lambda c: (c[0], c[1] is True, c[1] is None),
        )
        if not col_keys:
            continue

        # chenal first, then the rest alphabetically. chenal_32 is its own crate.
        ordered = []
        if "chenal" in crates:
            ordered.append("chenal")
        ordered.extend(sorted(c for c in crates if c != "chenal"))

        # Per-column baseline (chenal) and fastest crate.
        baselines = {}
        fastest = {}
        for col in col_keys:
            cells = columns[col]
            baselines[col] = cells.get("chenal")
            if cells:
                fastest[col] = min(cells, key=cells.get)

        out.append(f"### {kind} / {op}\n")

        header = "| crate | " + " | ".join(param_label(*c) for c in col_keys) + " |"
        sep = "| --- | " + " | ".join("---" for _ in col_keys) + " |"
        out.append(header)
        out.append(sep)

        for crate in ordered:
            row = [crate]
            for col in col_keys:
                cells = columns[col]
                ns = cells.get(crate)
                if ns is None:
                    row.append("—")
                    continue
                text = fmt_ns(ns)
                if fastest.get(col) == crate:
                    text = f"**{text}**"
                base = baselines[col]
                if base:
                    text += f" ({ns / base:.2f}×)"
                row.append(text)
            out.append("| " + " | ".join(row) + " |")

        out.append("")  # trailing blank line between tables

    return out


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    default_dir = os.path.join(script_dir, "target", "criterion")

    parser = argparse.ArgumentParser(
        description="Render chenal-centric markdown comparison tables from "
        "criterion benchmark results.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--criterion-dir",
        default=default_dir,
        help="Path to the criterion results directory.",
    )
    args = parser.parse_args()

    criterion_dir = os.path.abspath(args.criterion_dir)
    results = collect(criterion_dir)

    lines = [
        "# chenal channel benchmarks",
        "",
        "Comparison of `chenal` against competing channel crates. All values are "
        "**ns per message** (lower is faster).",
        "",
        "The parenthesized ratio is relative to `chenal` in the same column: "
        "`2.00×` means 2× slower than chenal, `1.00×` is the chenal baseline. "
        "The fastest crate in each column is **bold**. `—` means no result.",
        "",
    ]

    if not results:
        lines.append(f"_No valid benchmark results found under `{criterion_dir}`._")
    else:
        lines.extend(build_tables(results))

    print("\n".join(lines).rstrip() + "\n", end="")


if __name__ == "__main__":
    main()
