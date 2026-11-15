#!/usr/bin/env bash
#
# Compose check for the development stacks under .devcontainer/ (.devcontainer/README.md).
#
# A stack a developer starts on the host holds personal data behind no credential worth the name,
# so it publishes each port on a host address written in its Compose file, never on every
# interface of the host. Compose publishes a port without a host address on every interface, and
# one whose address is empty or 0.0.0.0 there too. This check reads every Compose file under
# .devcontainer/, hidden subdirectories included — a file named compose.yaml, compose.yml,
# docker-compose.yaml, or docker-compose.yml, or one of those names with a further part before the
# extension, such as compose.override.yaml — and fails on:
#
#   1. Unbound: a port entry of a 'ports:' list without a host address: "4318", "4318:4318", or
#      the long syntax without 'host_ip'.
#   2. Every interface: a host address that is empty, an IPv4 address of zeros, or an IPv6
#      address of zeros, such as ::, [::0], or [0:0:0:0:0:0:0:0], or of the IPv4 address it
#      maps after '::ffff:'.
#   3. Unset: a host address taken from a variable that can be empty: '$VAR' or '${VAR}', which
#      an unset variable empties; '${VAR-x}', which a variable set to nothing empties;
#      '${VAR:?msg}' and '${VAR:+x}', which write no address. '${VAR:-<address>}' passes when
#      its default passes rules 2 to 4.
#   4. Unread: what the check cannot decide, rather than passing it unseen — a host address that
#      is neither an IP address nor such a variable, a host name included; a 'ports:' value other
#      than a list in place or a flow list on its line, such as an alias or a flow list over
#      several lines; an entry written as a flow mapping; a line of a list that is no entry.
#   5. Host network: 'network_mode: host', which publishes every port of the service on every
#      interface without a 'ports:' entry, and a network mode taken from a variable.
#
# A port entry is read in the short syntax, '[<host>:]<published>:<target>[/<protocol>]',
# quoted or not, one entry a line or as a flow list on the 'ports:' line, after any anchor or tag
# such as '!override'; '!reset' publishes nothing and is skipped. In the long syntax, an entry is
# a list item of keys, one a line, whose 'host_ip' is the host address. A trailing '#' comment and
# a carriage return at the end of a line are dropped. A Compose file of another name is not read;
# no Compose file joins the Dev Container itself, since it adds no 'dockerComposeFile'. The files
# are small and hand-written, and no YAML parser is guaranteed on PATH.
#
# Pure bash + coreutils/find. Usage:
#     scripts/check-compose.sh [--root <dir>]
# The root defaults to this repository; scripts/test-check-compose.sh passes fixtures.
# Exit code 0 when the check passes, 1 otherwise, and 2 on an argument it does not read or a root
# that is no directory.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/bash.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/bash.sh"

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/check.sh"

# shellcheck source-path=SCRIPTDIR source=lib/root.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/root.sh"
usage="scripts/check-compose.sh [--root <dir>]"
enter_root "${BASH_SOURCE[0]}" "$usage" "$@"
rest_args "${BASH_SOURCE[0]}" "$usage" 0 "${ARGS[@]}"
DIR=".devcontainer"

# unquoted <text> — the text without surrounding blanks and one pair of surrounding quotes.
unquoted() {
  local t="$1"
  t="${t#"${t%%[![:space:]]*}"}"
  t="${t%"${t##*[![:space:]]}"}"
  if [[ "$t" =~ ^\"(.*)\"$ || "$t" =~ ^\'(.*)\'$ ]]; then t="${BASH_REMATCH[1]}"; fi
  printf '%s' "$t"
}

# uncommented <line> — the line without a trailing '#' comment, one after a blank outside quotes.
uncommented() {
  local line="$1" out="" c quote="" prev=" " i
  for ((i = 0; i < ${#line}; i++)); do
    c="${line:i:1}"
    if [[ -z "$quote" && "$c" == "#" && "$prev" == [[:space:]] ]]; then break; fi
    if [[ -z "$quote" && ( "$c" == '"' || "$c" == "'" ) ]]; then quote="$c"
    elif [[ "$c" == "$quote" ]]; then quote=""
    fi
    out+="$c" prev="$c"
  done
  printf '%s' "$out"
}

# address <where> <entry> <host> — judge one host address as written.
address() {
  local where="$1" entry="$2" host="$3" default bare
  local every="$where: the port \"$entry\" publishes on every interface of the host — name an address such as 127.0.0.1 [compose/port:every]"
  if [[ -z "$host" ]]; then
    add_error "$every"
  elif [[ "$host" =~ ^\$\{[A-Za-z_][A-Za-z0-9_]*:-(.*)\}$ ]]; then
    default="${BASH_REMATCH[1]}"
    if [[ "$default" == *'$'* ]]; then
      add_error "$where: the port \"$entry\" takes its address from a default the check cannot read [compose/port:unread]"
    else
      address "$where" "$entry" "$default"
    fi
  elif [[ "$host" =~ ^\$(\{[A-Za-z_][A-Za-z0-9_]*([-?+:].*)?\}|[A-Za-z_][A-Za-z0-9_]*)$ ]]; then
    add_error "$where: the port \"$entry\" takes its address from a variable that can be empty, which publishes it on every interface — give it a default, '\${VAR:-127.0.0.1}' [compose/port:unset]"
  elif [[ "$host" =~ ^[0-9]{1,3}(\.[0-9]{1,3}){3}$ ]]; then
    [[ "$host" =~ ^0+\.0+\.0+\.0+$ ]] && add_error "$every"
  elif [[ "$host" =~ ^\[([0-9A-Fa-f.]*:[0-9A-Fa-f:.]*)\]$ || "$host" =~ ^([0-9A-Fa-f.]*:[0-9A-Fa-f:.]*)$ ]]; then
    bare="${BASH_REMATCH[1]}"
    if [[ "${bare,,}" == ::ffff:* ]]; then
      address "$where" "$entry" "${bare:7}"
    elif [[ "$bare" =~ ^[0:.]+$ ]]; then
      add_error "$every"
    fi
  else
    add_error "$where: the port \"$entry\" has a host address the check cannot read, neither an IP address nor a variable with one as its default [compose/port:unread]"
  fi
}

# short <where> <entry> — judge one entry of the short syntax.
short() {
  local where="$1" entry="$2" masked="$2" host n
  local -a held fields
  # Hide the colons inside variables and IPv6 brackets, then split at the rest.
  while [[ "$masked" =~ (\$\{[^}]*\}|\[[^]]*\]) ]]; do
    held+=("${BASH_REMATCH[1]}")
    masked="${masked/"${BASH_REMATCH[1]}"/$'\x1f'$((${#held[@]} - 1))$'\x1f'}"
  done
  IFS=: read -r -a fields <<< "$masked"
  [[ "$masked" == *: ]] && fields+=("")
  if ((${#fields[@]} < 3)); then
    add_error "$where: the port \"$entry\" names no host address, so it is published on every interface of the host [compose/port:unbound]"
    return
  fi
  host="${masked%:*:*}"
  for n in "${!held[@]}"; do host="${host//$'\x1f'$n$'\x1f'/${held[n]}}"; done
  address "$where" "$entry" "$host"
}

# long_end — judge the long-syntax entry read so far, if any.
long_end() {
  ((long_line)) || return
  if [[ -z "$long_ip_set" ]]; then
    add_error "$name:$long_line: the port \"$long_desc\" names no host_ip, so it is published on every interface of the host [compose/port:unbound]"
  else
    address "$name:$long_line" "$long_desc" "$long_ip"
  fi
  long_line=0
}

shopt -s nullglob globstar dotglob
files=()
for f in "$ROOT/$DIR"/**/*; do
  [[ -f "$f" ]] || continue
  [[ "${f##*/}" =~ ^(docker-)?compose(\.[^/]+)?\.ya?ml$ ]] && files+=("$f")
done

entries=0
for file in "${files[@]}"; do
  name="${file#"$ROOT"/}"
  line_no=0 ports_indent=-1 item_indent=-1 long_line=0 long_ip="" long_ip_set="" long_desc=""
  while IFS= read -r raw || [[ -n "$raw" ]]; do
    line_no=$((line_no + 1))
    line="$(uncommented "${raw%$'\r'}")"
    line="${line%"${line##*[![:space:]]}"}"
    [[ "$line" =~ ^[[:space:]]*$ ]] && continue
    indent="${line%%[![:space:]]*}" indent="${#indent}"
    text="${line#"${line%%[![:space:]]*}"}"

    if [[ "$text" =~ ^[\"\']?network_mode[\"\']?:(.*)$ ]]; then
      mode="$(unquoted "${BASH_REMATCH[1]}")"
      if [[ "$mode" == host ]]; then
        add_error "$name:$line_no: 'network_mode: host' publishes every port of the service on every interface of the host [compose/network:host]"
      elif [[ "$mode" == *'$'* ]]; then
        add_error "$name:$line_no: the network mode \"$mode\" comes from a variable, which can name the host's network [compose/network:unread]"
      fi
    fi

    if ((ports_indent >= 0)); then
      if ((indent > ports_indent)) || [[ $indent -eq $ports_indent && "$text" == '- '* ]]; then
        if [[ "$text" == '-'* ]] && ((item_indent < 0 || indent == item_indent)); then
          long_end
          item_indent=$indent
          item="$(unquoted "${text#-}")"
          if [[ "$item" == '{'* ]]; then
            entries=$((entries + 1))
            add_error "$name:$line_no: the port \"$item\" is a flow mapping, which the check cannot read — write the long syntax one key a line [compose/port:unread]"
            continue
          elif [[ -z "$item" || "$item" =~ ^[A-Za-z_]+:([[:space:]]|$) ]]; then
            long_line=$line_no long_ip="" long_ip_set="" long_desc="${item:-long syntax}"
            entries=$((entries + 1))
            [[ "$item" =~ ^host_ip:(.*)$ ]] && long_ip="$(unquoted "${BASH_REMATCH[1]}")" long_ip_set=1
            continue
          else
            entries=$((entries + 1))
            short "$name:$line_no" "$item"
            continue
          fi
        fi
        if ((!long_line || indent <= item_indent)); then
          add_error "$name:$line_no: the line \"$text\" of a ports list is no entry the check can read [compose/port:unread]"
          continue
        fi
        if [[ "$text" =~ ^host_ip:(.*)$ ]]; then
          long_ip="$(unquoted "${BASH_REMATCH[1]}")" long_ip_set=1
        fi
        [[ "$text" =~ ^(target|published): && "$long_desc" != *"$text"* ]] && long_desc+=", $text"
        continue
      fi
      long_end
      ports_indent=-1
    fi

    if [[ "$text" =~ ^[\"\']?ports[\"\']?:[[:space:]]*(.*)$ ]]; then
      rest="${BASH_REMATCH[1]}" reset=""
      while [[ "$rest" =~ ^([\&!][^[:space:]]+)[[:space:]]*(.*)$ ]]; do
        [[ "${BASH_REMATCH[1]}" == '!reset' ]] && reset=1
        rest="${BASH_REMATCH[2]}"
      done
      if [[ -n "$reset" ]]; then
        :
      elif [[ -z "$rest" ]]; then
        ports_indent=$indent item_indent=-1
      elif [[ "$rest" =~ ^\[([^{]*)\]$ ]]; then
        IFS=, read -r -a flow <<< "${BASH_REMATCH[1]}"
        for item in "${flow[@]}"; do
          item="$(unquoted "$item")"
          [[ -n "$item" ]] || continue
          entries=$((entries + 1))
          short "$name:$line_no" "$item"
        done
      else
        add_error "$name:$line_no: the ports \"$rest\" are in a form the check cannot read, such as an alias or a flow list over several lines — write the list in place [compose/port:unread]"
      fi
    fi
  done < "$file"
  long_end
done

if ((${#errors[@]})); then
  echo "Compose check FAILED:" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi
echo "Compose check passed (${#files[@]} files, $entries ports, each on a named host address) [compose/check:passed]"
exit 0
