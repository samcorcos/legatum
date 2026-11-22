// Sound board: every feel event with its sound, source and license, to judge by ear.
// Verdicts and notes post to /api/feedback as item "sound:<event>" for the lead to iterate on.
(() => {
  const box = document.getElementById("sound-board");
  if (!box) return;
  const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
  let manifest = [];
  let verdicts = {};
  let playing = null;

  function stop() {
    if (!playing) return;
    playing.el.pause();
    playing.button.textContent = playing.label;
    playing = null;
  }

  // One element at a time; music toggles, "seam" jumps to 4 s before the loop point.
  function play(button, file, { loop = false, seam = false } = {}) {
    const same = playing && playing.button === button;
    stop();
    if (same) return;
    const el = new Audio(`/audio/${file}`);
    el.loop = loop;
    const label = button.textContent;
    playing = { el, button, label };
    if (seam) el.addEventListener("loadedmetadata", () => { el.currentTime = Math.max(0, el.duration - 4); }, { once: true });
    el.addEventListener("ended", () => { if (playing && playing.el === el) stop(); });
    button.textContent = "■ Stop";
    el.play().catch(() => stop());
  }

  async function send(event, reaction) {
    const note = box.querySelector(`input[data-note="${CSS.escape(event)}"]`);
    const text = note ? note.value : "";
    if (!text.trim() && !reaction) return;
    const r = await fetch("/api/feedback", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ item: `sound:${event}`, text, reaction }) });
    const status = box.querySelector(`[data-status="${CSS.escape(event)}"]`);
    if (r.ok) {
      if (note) note.value = "";
      verdicts[event] = { reaction, text, t: new Date().toISOString() };
      if (status) status.textContent = reaction ? `Sent: ${reaction}` : "Note sent";
    } else if (status) status.textContent = "Not sent";
  }

  function row(entry, event) {
    const music = event.startsWith("music:");
    const v = verdicts[event];
    const last = v ? `${v.reaction || ""}${v.text ? ` “${v.text}”` : ""}` : "";
    return `<tr>
      <td><b>${esc(music ? `Music · ${event.slice(6)} look` : event)}</b><div class="sb-why">${esc(entry.rationale)}</div></td>
      <td class="sb-play"><button data-play="${esc(entry.file)}" data-loop="${music}">▶ Play</button>${music ? `<button data-play="${esc(entry.file)}" data-seam="1" title="Play across the loop point">▶ Seam</button>` : ""}</td>
      <td><code>${esc(entry.file)}</code><div class="sb-src"><a href="${esc(entry.source)}" target="_blank" rel="noreferrer">${esc(entry.title)}</a></div></td>
      <td>${esc(entry.author)}<div class="sb-src">${esc(entry.license)}</div></td>
      <td class="sb-verdict">
        <div class="actions"><button data-verdict="${esc(event)}" data-reaction="👍 Good">👍</button><button data-verdict="${esc(event)}" data-reaction="👎 Not right">👎</button>
        <input data-note="${esc(event)}" placeholder="Note (optional)" /><button data-verdict="${esc(event)}" data-reaction="">Send</button></div>
        <div class="sent" data-status="${esc(event)}">${esc(last)}</div>
      </td>
    </tr>`;
  }

  function render() {
    const rows = manifest.flatMap((entry) => entry.events.map((event) => row(entry, event)));
    box.innerHTML = `<div class="sb-note">${rows.length} events. Headphones or speakers at a normal level; the game plays these at about -20 LUFS (music -18). Technical numbers: docs/sound-qa.md.</div>
      <div class="sb-scroll"><table class="sb-table"><thead><tr><th>Event</th><th></th><th>File / source</th><th>Author / license</th><th>Your verdict</th></tr></thead><tbody>${rows.join("")}</tbody></table></div>`;
    document.getElementById("sound-count").textContent = `${rows.length}`;
  }

  box.addEventListener("click", (e) => {
    const b = e.target.closest("button");
    if (!b) return;
    if (b.dataset.play) play(b, b.dataset.play, { loop: b.dataset.loop === "true" || !!b.dataset.seam, seam: !!b.dataset.seam });
    else if (b.dataset.verdict) send(b.dataset.verdict, b.dataset.reaction);
  });
  box.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && e.target.dataset.note) send(e.target.dataset.note, "");
  });

  async function load() {
    try {
      manifest = await (await fetch("/audio/manifest.json", { cache: "no-store" })).json();
      const state = await (await fetch("/api/state", { cache: "no-store" })).json().catch(() => ({}));
      for (const f of state.feedback || []) if (String(f.item).startsWith("sound:")) verdicts[f.item.slice(6)] = f;
      render();
    } catch {
      box.innerHTML = '<div class="empty">No sound manifest (app/public/audio/manifest.json).</div>';
    }
  }
  load();
})();
