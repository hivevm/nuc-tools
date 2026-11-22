# shellcheck shell=bash
# The replay of the take-ups, sourced by scripts/check-parity.sh.
# Verifies: API-upgrade-by-blob-toml-optional, G-4
#
# Each row of tests/parity/take-ups.tsv names a take-up a human committed: the repository by URL,
# the commit before the take-up and the commit that ends it, each by its 40-hex name, and the URL
# of the template. The old release is the one the README names at the first commit, the new one
# the one it names at the second. The replay fetches both commits, and the two tags of the
# template into refs/template/, into a repository of its own in the cache, checks the first commit
# out on a branch of its own in a directory of the harness's, runs 'nuc upgrade <new>' there, and
# compares what it wrote with the take-up.
#
# Compared per take-up: each path 'nuc upgrade' writes or deletes, as git's status lists it, the
# files its ignore rules leave out included, holds the bytes and the executable bit the second
# commit holds there, or is missing where that commit lacks it. A path it lists as a judgment, a
# line of its standard output, is not compared. A path that differs fails the replay and is named,
# unless tests/parity/take-up-differences.tsv lists it for the new release with a reason; a row of
# that table no replay meets fails as well, so that a stale reason does not stand. 'nuc upgrade'
# exiting 2 or more, a commit or a tag that cannot be fetched, a release line that names no
# release, and a replay that cannot be set up fail, each with git's reason; none is skipped (Q-1).
#
# PARITY_TAKE_UPS and PARITY_TAKE_UP_DIFFERENCES name other tables, for
# scripts/test-check-parity.sh.

# The rows of the two tables, as take_up_load reads them.
TAKE_UP_ROWS=()
declare -A TAKE_UP_KNOWN=()

# take_up_load — the two tables read and checked, before any comparison runs; 2 on a missing or
# malformed table, which names it.
take_up_load() {
  local table="${PARITY_TAKE_UPS:-$ROOT/tests/parity/take-ups.tsv}"
  local differences="${PARITY_TAKE_UP_DIFFERENCES:-$ROOT/tests/parity/take-up-differences.tsv}"
  local row url from to template release path reason extra hex='^[0-9a-f]{40}$'
  TAKE_UP_ROWS=() TAKE_UP_KNOWN=()
  [[ -f "$table" ]] || { echo "Parity check: $table — not found" >&2; return 2; }
  [[ -f "$differences" ]] || { echo "Parity check: $differences — not found" >&2; return 2; }
  while IFS= read -r row || [[ -n "$row" ]]; do
    [[ -z "$row" || "$row" == "#"* ]] && continue
    IFS=$'\t' read -r release path reason extra <<< "$row"
    if ! [[ "$release" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ && -n "$path" && -n "$reason" && -z "$extra" ]]; then
      echo "Parity check: $differences — the line '$row' is not '<release><TAB><path><TAB><reason>'" >&2
      return 2
    fi
    TAKE_UP_KNOWN["$release $path"]="$reason"
  done < "$differences"
  while IFS= read -r row || [[ -n "$row" ]]; do
    [[ -z "$row" || "$row" == "#"* ]] && continue
    IFS=$'\t' read -r url from to template extra <<< "$row"
    if ! [[ -n "$url" && "$from" =~ $hex && "$to" =~ $hex && -n "$template" && -z "$extra" ]]; then
      echo "Parity check: $table — the line '$row' is not '<url><TAB><commit><TAB><commit><TAB><template url>'" >&2
      return 2
    fi
    TAKE_UP_ROWS+=("$row")
  done < "$table"
}

# take_up_replays <nuc> <work> <cache> — each take-up take_up_load read, replayed; 0 where each
# agrees, 1 where one differs or cannot be replayed.
take_up_replays() {
  local nuc="$1" work="$2" cache="$3" failed=0
  local repo="$cache/.take-ups"
  local differences="${PARITY_TAKE_UP_DIFFERENCES:-$ROOT/tests/parity/take-up-differences.tsv}"
  # The binary runs in the directory of each replay, so a path given relative is made absolute.
  [[ "$nuc" == /* ]] || nuc="$PWD/$nuc"
  declare -A met=()
  local row url from to template old new dir out err rc judged entry status path how count want
  mkdir -p "$work/take-ups"
  local why="$work/take-ups/git.err"
  if ((${#TAKE_UP_ROWS[@]})) && ! { [[ -d "$repo/.git" ]] || { mkdir -p "$repo" && cgit "$repo" init -q; }; }; then
    echo "Parity check: $repo — the cache of the take-ups cannot be made" >&2
    return 1
  fi
  for row in "${TAKE_UP_ROWS[@]}"; do
    IFS=$'\t' read -r url from to template <<< "$row"
    if ! { cgit "$repo" fetch -q --depth 1 "$url" "$from" "$to" \
      && cgit "$repo" update-ref "refs/take-ups/$from" "$from" \
      && cgit "$repo" update-ref "refs/take-ups/$to" "$to"; } 2> "$why"; then
      echo "Parity check: take-up ${to:0:12} — its commits cannot be fetched from $url: $(tail -n 1 "$why")" >&2
      failed=1
      continue
    fi
    old="$(release_of <(cgit "$repo" show "$from:README.md" 2> /dev/null))"
    new="$(release_of <(cgit "$repo" show "$to:README.md" 2> /dev/null))"
    if [[ -z "$old" || -z "$new" ]]; then
      echo "Parity check: take-up ${to:0:12} — its README names no release at ${from:0:12} or ${to:0:12}" >&2
      failed=1
      continue
    fi
    if ! cgit "$repo" fetch -q --depth 1 "$template" \
      "+refs/tags/$old:refs/tags/$old" "+refs/tags/$new:refs/tags/$new" 2> "$why"; then
      echo "Parity check: take-up $old -> $new — the tags cannot be fetched from $template: $(tail -n 1 "$why")" >&2
      failed=1
      continue
    fi
    dir="$work/take-ups/$new"
    rm -rf "$dir"
    if ! { mkdir -p "$dir" && cgit "$dir" init -q && cgit "$dir" config core.autocrlf false \
      && cgit "$dir" fetch -q --no-tags --update-shallow "$repo" "refs/take-ups/$from:refs/heads/take-up" \
        "refs/take-ups/$to:refs/take-up/end" "refs/tags/$old:refs/template/$old" \
        "refs/tags/$new:refs/template/$new" \
      && cgit "$dir" checkout -q take-up; } 2> "$why"; then
      echo "Parity check: take-up $old -> $new — its replay could not be set up: $(tail -n 1 "$why")" >&2
      failed=1
      continue
    fi
    out="$work/take-ups/$new.out" err="$work/take-ups/$new.err"
    (cd "$dir" && "$nuc" upgrade "$new") > "$out" 2> "$err"
    rc=$?
    if ((rc >= 2)); then
      echo "Parity check: take-up $old -> $new — nuc upgrade exits $rc: $(head -n 1 "$err")" >&2
      failed=1
      continue
    fi
    # The paths nuc upgrade lists: the file of each finding, its line left off.
    judged="$(sed -E 's/: upgrade\/[a-z]+: .*//; s/:[0-9]+$//' "$out" | sort -u)"
    count=0
    while IFS= read -r -d '' entry; do
      status="${entry:0:2}" path="${entry:3}"
      grep -qxF -- "$path" <<< "$judged" && continue
      count=$((count + 1))
      if cgit "$dir" cat-file -e "refs/take-up/end:$path" 2> /dev/null; then
        if [[ ! -e "$dir/$path" ]]; then
          how="is deleted where the take-up keeps it"
        elif ! cmp -s "$dir/$path" <(cgit "$dir" cat-file blob "refs/take-up/end:$path"); then
          how="holds other bytes than the take-up"
        else
          want="$(cgit "$dir" ls-tree refs/take-up/end -- "$path" | cut -d ' ' -f 1)"
          [[ "$want" == 100755 && -x "$dir/$path" ]] && continue
          [[ "$want" != 100755 && ! -x "$dir/$path" ]] && continue
          how="has another executable bit than the take-up"
        fi
      else
        [[ -e "$dir/$path" || -L "$dir/$path" ]] || continue
        how="is written where the take-up has none"
      fi
      if [[ -n "${TAKE_UP_KNOWN["$new $path"]+set}" ]]; then
        met["$new $path"]=1
        continue
      fi
      echo "Parity check: take-up $old -> $new — $path ($status) $how, and ${differences#"$ROOT/"} does not list it" >&2
      failed=1
    done < <(cgit "$dir" status --porcelain -z --untracked-files=all --ignored=matching)
    echo "take-up $old -> $new at ${to:0:12}: $count path(s) written or deleted, $(grep -c . <<< "$judged") listed"
  done
  for entry in "${!TAKE_UP_KNOWN[@]}"; do
    [[ -n "${met["$entry"]+set}" ]] && continue
    echo "Parity check: ${differences#"$ROOT/"} — '${entry/ /: }' is listed, and no replay writes it otherwise than its take-up" >&2
    failed=1
  done
  return "$failed"
}
