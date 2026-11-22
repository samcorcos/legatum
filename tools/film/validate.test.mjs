// node --test tools/film/
import assert from "node:assert/strict";
import { test } from "node:test";
import { claimsIn, checkSentence, collectFacts, validateScript, splitSentences } from "./lib/validate.mjs";

const facts = {
  run_id: "2026-09-27-2330",
  generations: {
    count: 3, first: 0, last: 20,
    series: [
      { gen: 0, proxy_win_rate: 0.5, best_fitness: 1.25, archetype_share: { Rush: 0.125, Science: 0.25 } },
      { gen: 10, proxy_win_rate: 0.25, best_fitness: 1.25, archetype_share: { Rush: 0.161, Science: 0.214 } },
      { gen: 20, proxy_win_rate: 0.375, best_fitness: 1.25, archetype_share: { Rush: 0.179, Science: 0.179 } },
    ],
  },
  coach: [{ gen: 10, summary: "production_idle went from 4.1% to 0.6% of opportunities.", observations: [{ evidence: [{ game_id: "native-1790551705787859000-78448-7", turn: 45, player: 2, detail: "Japan: production empty" }] }] }],
  checkpoints: [{ gen: 20, showcase: [{ game_id: "native-1790552309993411000-98709-1", seed: "880002", highlight_turns: [{ turn: 141, player: 2, note: "Byzantium wins with 10 VP" }] }] }],
};
const known = collectFacts(facts);

test("claims carry the kind of number they state", () => {
  const c = claimsIn("By generation 10 the Proxy won 25% of games, down from 50 percent, on turn 141.");
  assert.deepEqual(c.map((x) => [x.kind, x.value]), [["gen", 10], ["percent", 25], ["percent", 50], ["turn", 141]]);
  assert.deepEqual(claimsIn("Generations 0–20 and turns 45 to 141").map((x) => x.kind), ["gen", "gen", "turn", "turn"]);
  assert.deepEqual(claimsIn("forty percent").map((x) => [x.kind, x.value]), [["percent", 40]]);
});

test("supported numbers pass", () => {
  for (const s of [
    "At generation 0 the Proxy won 50% of its games.",
    "By generation 20 it won 37.5% of them.",
    "Idle production fell from 4.1% to 0.6%.",
    "Watch turn 141 of game native-1790552309993411000-98709-1.",
    "The best candidate scored 1.25.",
    "Two strategies traded places.",
  ]) assert.equal(checkSentence(s, known).ok, true, s);
});

test("rounded percentages match within half a point", () => {
  assert.equal(checkSentence("The Proxy won 38% of games at generation 20.", known).ok, true);
  assert.equal(checkSentence("The Proxy won 40% of games at generation 20.", known).ok, false);
});

test("invented numbers, generations, turns and ids are rejected", () => {
  assert.equal(checkSentence("By generation 15 the AI learned to retreat.", known).ok, false);
  assert.equal(checkSentence("The Proxy won 90% of its games.", known).ok, false);
  assert.equal(checkSentence("On turn 99 the Japan rushed.", known).ok, false);
  assert.equal(checkSentence("Game native-1234567890-1-1 shows it.", known).ok, false);
  assert.equal(checkSentence("It took 1,200 games.", known).ok, false);
  assert.equal(checkSentence("By generation 2330 the AI had changed.", known).ok, false, "a run id suffix is not a generation");
});

test("a generation number is not accepted as a percentage and vice versa", () => {
  assert.equal(checkSentence("It won 20% of games.", known).ok, false);
  assert.equal(checkSentence("By generation 25 it was over.", known).ok, false);
});

test("validateScript drops unsupported sentences and scenes on unknown tapes", () => {
  const script = {
    scenes: [
      { type: "title", params: { heading: "What the AI learned" }, narration: [] },
      { type: "graph", params: { metric: "proxy_win_rate", gen_from: 0, gen_to: 21 }, narration: ["The Proxy won 50% at first. Then it won 3% by generation 20."] },
      { type: "map", params: { game_id: "native-unknown-000000-1" }, narration: ["Look at this game."] },
      { type: "text", params: { heading: "It won 99% of games" }, narration: ["It won 99% of games."] },
    ],
  };
  const { script: out, report } = validateScript(script, facts, { mapGames: new Set(["native-1790552309993411000-98709-1"]) });
  assert.deepEqual(out.scenes.map((s) => s.type), ["title", "graph"]);
  assert.deepEqual(out.scenes[1].narration, ["The Proxy won 50% at first."]);
  assert.equal(out.scenes[1].params.gen_to, 20);
  assert.ok(report.some((r) => r.includes("3%")));
  assert.ok(report.some((r) => r.includes("not a showcase tape")));
});

test("sentences split on terminal punctuation only", () => {
  assert.deepEqual(splitSentences("It won 37.5% of games. Then it lost."), ["It won 37.5% of games.", "Then it lost."]);
});
