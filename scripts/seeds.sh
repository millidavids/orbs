#!/usr/bin/env bash
# Train the readers over several seeds and measure every run, so a change can be
# told from the luck of one.
#
# **One run cannot tell a fix from a seed.** The trainer draws the embedding
# table first, so any change to the vocabulary's size moves every weight's
# starting value — and on `wgpu` a seed does not even reproduce itself bit for
# bit. DESIGN.md §19 has a retrain that lost four points on a reader whose
# corpus it never touched. A change is real when it clears the spread these runs
# measure, not when one run of it beats one run of what it replaced.
#
#     scripts/seeds.sh baseline                  # both readers, seeds 181 1 2 3 4
#     scripts/seeds.sh fold                      # ...the same, after a change
#     scripts/seeds.sh counts --spells baseline  # the spell reader alone; its
#                                                # command lines read by the
#                                                # baseline's prompt reader of
#                                                # the same seed
#     scripts/seeds.sh --compare baseline fold   # did it clear the spread?
#     scripts/seeds.sh --summary baseline        # print a run's summary again
#     SEEDS="181 1 2" scripts/seeds.sh baseline  # fewer, for a quick look
#
# **Five seeds, because three understate the spread.** The range of five runs is
# wider than the range of three drawn from the same luck, so a change that clears
# it has cleared more of what a seed can do on its own — and at eight seconds an
# epoch, five runs of both readers are minutes rather than an afternoon.
#
# Everything lands in `target/seeds/<label>/` (`SEEDS_DIR` moves it): each
# seed's weights, the trainer's log, one `.scores` file per measurement, the
# binaries that produced them, and `summary.txt` — every score's mean and range
# across the seeds.
#
# **Ship what was measured.** A retrain is a new run, so keeping one of these
# means copying its `.bin` over `crates/orbs-augury/weights/`, not training it
# again. Seed 181 is the trainer's default, the one a plain `train` takes.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
runs="${SEEDS_DIR:-target/seeds}"

# The seeds a label's scores were measured over, from the files themselves.
seeds_in() {
  local file seed
  for file in "$1"/*.scores; do
    [[ -e "$file" ]] || continue
    seed="${file##*-}"
    echo "${seed%.scores}"
  done | sort -un | tr '\n' ' '
}

# Every score's mean and range across the seeds, in the order the runs print them.
summarise() {
  local out="$1" kind
  echo "$(basename "$out") — seeds $(seeds_in "$out")"
  for kind in prompt spells trials; do
    compgen -G "$out/$kind-*.scores" > /dev/null || continue
    echo "  $kind"
    awk -F'\t' '
      !($1 in count) { order[++names] = $1 }
      {
        count[$1]++; sum[$1] += $2
        if (count[$1] == 1 || $2 < low[$1]) low[$1] = $2
        if (count[$1] == 1 || $2 > high[$1]) high[$1] = $2
      }
      END {
        for (i = 1; i <= names; i++) {
          name = order[i]
          printf "    %-32s %6.1f   %6.1f to %-6.1f  spread %4.1f\n",
            name, sum[name] / count[name], low[name], high[name], high[name] - low[name]
        }
      }' "$out/$kind"-*.scores
  done
}

# Two labels side by side, score by score. **A score moved only when one range
# clears the other** — `above` or `below`; anything that overlaps is `within`,
# which is what a seed can do on its own. Neutral words rather than better and
# worse, because for `wrongly refused` and `read as their opposite` lower is the
# good direction.
#
# **The Welch t beside it is a hint, never the verdict.** A change that touched
# nothing but the table's size reached 2.2 on one score of twenty — which is what
# twenty scores of luck produce (DESIGN.md §19, *Several seeds*).
compare() {
  local before="$runs/$1" after="$runs/$2" kind
  echo "$1 -> $2   (a score moved only when one range clears the other)"
  for kind in prompt spells trials; do
    compgen -G "$before/$kind-*.scores" > /dev/null || continue
    compgen -G "$after/$kind-*.scores" > /dev/null || continue
    echo "  $kind"
    awk -F'\t' -v before="$before/" '
      {
        side = index(FILENAME, before) == 1 ? "was" : "now"
        if (!($1 in seen)) { order[++names] = $1; seen[$1] = 1 }
        at = side SUBSEP $1
        count[at]++; sum[at] += $2; squares[at] += $2 * $2
        if (count[at] == 1 || $2 < low[at]) low[at] = $2
        if (count[at] == 1 || $2 > high[at]) high[at] = $2
      }
      # The sample variance, from the running sums; never below nought, which
      # rounding can otherwise reach on a score every seed agrees on.
      function spread(at,   n, mean, v) {
        n = count[at]; mean = sum[at] / n
        v = n > 1 ? (squares[at] - n * mean * mean) / (n - 1) : 0
        return v > 0 ? v : 0
      }
      END {
        for (i = 1; i <= names; i++) {
          name = order[i]; was = "was" SUBSEP name; now = "now" SUBSEP name
          if (!(was in count) || !(now in count)) continue
          moved = low[now] > high[was] ? "above" : high[now] < low[was] ? "below" : "within"
          error = sqrt(spread(was) / count[was] + spread(now) / count[now])
          t = error > 0 ? sprintf("%+6.2f", (sum[now] / count[now] - sum[was] / count[was]) / error) : "     -"
          printf "    %-32s %6.1f (%5.1f to %-5.1f) -> %6.1f (%5.1f to %-5.1f)  t %s  %s\n",
            name, sum[was] / count[was], low[was], high[was],
            sum[now] / count[now], low[now], high[now], t, moved
        }
      }' "$before/$kind"-*.scores "$after/$kind"-*.scores
  done
}

case "${1:-}" in
  --summary)
    summarise "$runs/${2:?usage: seeds.sh --summary <label>}"
    exit 0
    ;;
  --compare)
    compare "${2:?usage: seeds.sh --compare <before> <after>}" \
      "${3:?usage: seeds.sh --compare <before> <after>}"
    exit 0
    ;;
esac

label="${1:?usage: seeds.sh <label> [--spells <label-of-prompt-readers>]}"
shift
prompts_from=""
if [[ "${1:-}" == "--spells" ]]; then
  prompts_from="${2:?--spells needs the label whose prompt readers read the command lines}"
fi
seeds="${SEEDS:-181 1 2 3 4}"
out="$runs/$label"
mkdir -p "$out"
# **A label is one run.** `summarise` and `compare` glob every `$kind-*.scores`
# in the directory, so re-running a five-seed label over three seeds would fold
# the old tree's seeds 3 and 4 into this run's mean, range and t — a contaminated
# spread, reported as this change's. The weights and logs stay: `--spells
# <label>` reads the prompt readers back out of them.
rm -f "$out"/*.scores "$out/summary.txt"

cargo build -q --release -p orbs-augury --features train \
  --example train --example measure --example trials
# **Copied, not run in place.** A run takes hours and the tree is not frozen for
# them, so a build made meanwhile — the next change's experiment — would
# otherwise train the later seeds with a different trainer than the first. The
# copies carry their corpus with them (it is compiled in), and they are the
# record of what produced these numbers.
bin="$out/bin"
mkdir -p "$bin"
cp target/release/examples/train target/release/examples/measure \
  target/release/examples/trials "$bin/"

# The trainer's own selection score — its holdout's class, tag and refusal mean —
# out of its log and into a run's scores, beside what `measure` makes of it.
trained() {
  sed -n 's/.*refusal mean: \([0-9.]*\)%.*/trainer\t\1/p' "$1"
}

for seed in $seeds; do
  if [[ -z "$prompts_from" ]]; then
    reader="$out/reader-$seed"
    "$bin/train" --seed "$seed" --out "$reader" > "$reader.log" 2>&1
    { trained "$reader.log"; "$bin/measure" --reader "$reader" --scores; } \
      > "$out/prompt-$seed.scores"
  else
    # **The same prompt reader, seed for seed**, so a spell-only change is
    # compared with the command lines read exactly as the baseline read them.
    reader="$runs/$prompts_from/reader-$seed"
    if [[ ! -f "$reader.bin" ]]; then
      echo "no $reader.bin — run scripts/seeds.sh $prompts_from first" >&2
      exit 1
    fi
  fi
  scribe="$out/scribe-$seed"
  "$bin/train" --spells --seed "$seed" --out "$scribe" > "$scribe.log" 2>&1
  { trained "$scribe.log"; "$bin/measure" --spells --scribe "$scribe" --reader "$reader" --scores; } \
    > "$out/spells-$seed.scores"
  "$bin/trials" --scribe "$scribe" --reader "$reader" --scores > "$out/trials-$seed.scores"
  echo "  seed $seed measured"
done

summarise "$out" | tee "$out/summary.txt"
