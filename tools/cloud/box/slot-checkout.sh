#!/usr/bin/env bash
# Idempotently fetch a job's refs into a slot and check out its requested commit.
slot_fetch_checkout() {
  local repo="$1" job="$2" sha="$3" slot="$4" attempt
  for attempt in 1 2 3; do
    if git -C "$slot" fetch -q "$repo" "+refs/jobs/$job/*:refs/jobs/$job/*" \
      && git -C "$slot" reset -q --hard \
      && git -C "$slot" checkout -q -f --detach "$sha"; then
      return 0
    fi
    [ "$attempt" -lt 3 ] || break
    echo "box: retrying idempotent slot fetch/checkout ($attempt/3)" >&2
    sleep 1
  done
  return 1
}
