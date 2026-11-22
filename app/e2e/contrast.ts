import type { CDPSession, Page } from '@playwright/test';

/** One label of one control, measured in one state. */
export interface ContrastReading {
  /** Where: the control's classes and label. */
  where: string;
  /** default, hover, focus-visible, active, or disabled. */
  state: string;
  /** The text run measured inside the control (its element's classes). */
  text: string;
  ratio: number;
  /** WCAG AA minimum for this text: 4.5, or 3 for large text (24px, or 18.66px bold). */
  required: number;
  fg: string;
  bg: string;
}

const STATES = ['default', 'hover', 'focus-visible', 'active'] as const;

const CONTROLS = 'button, [role="button"], [role="option"], [role="radio"], [role="tab"], a.btn, summary, .era-card[tabindex]';

/**
 * Measures the label contrast of every distinct control on screen, in every interaction state,
 * from real pixels. For each state it forces the pseudo-class on every control (Chrome DevTools
 * `CSS.forcePseudoState`) and takes two screenshots: one with every label's glyphs thickened by a
 * stroke in their own colour (so glyph cores are solid and anti-aliasing can't understate the
 * ink), and one with the labels made transparent (the background exactly as drawn). Opacity,
 * filters, gradients and paintings behind the control are therefore all accounted for.
 *
 * Only controls in view are measured. Disabled controls are measured once, as the state
 * "disabled". Controls that look the same (same classes, state attributes and container) are
 * measured once.
 */
export async function auditContrast(page: Page, screen: string): Promise<ContrastReading[]> {
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('DOM.enable');
  await cdp.send('CSS.enable');
  await page.evaluate(() => {
    if (!document.getElementById('contrast-audit-style')) {
      const style = document.createElement('style');
      style.id = 'contrast-audit-style';
      style.textContent = `*, *::before, *::after { transition: none !important; caret-color: transparent !important; }
        .cx-thick { -webkit-text-stroke: 2px currentColor !important; }
        .cx-hide { color: transparent !important; -webkit-text-fill-color: transparent !important; -webkit-text-stroke: 0 !important; text-shadow: none !important; }`;
      document.head.appendChild(style);
    }
    // Entrance animations end in their final frame (fill-mode both), exactly as a player sees it.
    for (const a of document.getAnimations()) {
      try { a.finish(); } catch { /* an endless animation (a spinner) has no final frame */ }
    }
  });
  await tagControls(page);
  const readings: ContrastReading[] = [];
  try {
    for (const state of STATES) {
      // React can replace a control while the screenshots for a prior state are taken.
      const enabled = await nodeIdsFor(cdp, '[data-cx]:not(:disabled):not([aria-disabled="true"])');
      for (const nodeId of enabled) {
        try {
          await cdp.send('CSS.forcePseudoState', { nodeId, forcedPseudoClasses: state === 'default' ? [] : [state] });
        } catch (error) {
          // A replaced control has no pixels left to audit; measureAll reads only live tagged runs.
          if (!String(error).includes('Could not find node with given id')) throw error;
        }
      }
      readings.push(...await measureAll(page, state));
    }
  } finally {
    for (const nodeId of await nodeIdsFor(cdp, '[data-cx]').catch(() => [] as number[])) {
      await cdp.send('CSS.forcePseudoState', { nodeId, forcedPseudoClasses: [] }).catch(() => {});
    }
    await page.evaluate(() => {
      for (const el of document.querySelectorAll('[data-cx], [data-cx-run]')) {
        el.removeAttribute('data-cx');
        el.removeAttribute('data-cx-run');
        el.classList.remove('cx-thick', 'cx-hide');
      }
      document.getElementById('contrast-audit-style')?.remove();
    });
    await cdp.detach();
  }
  return readings.map((r) => ({ ...r, where: `${screen}: ${r.where}` }));
}

async function nodeIdsFor(cdp: CDPSession, selector: string) {
  const { root } = await cdp.send('DOM.getDocument', { depth: 0 });
  const { nodeIds } = await cdp.send('DOM.querySelectorAll', { nodeId: root.nodeId, selector });
  return nodeIds;
}

/** Tags each distinct control in view `data-cx`, and each of its text runs `data-cx-run`. */
async function tagControls(page: Page) {
  return page.evaluate((selector) => {
    const modal = [...document.querySelectorAll('dialog')].filter((d) => d.matches(':modal')).at(-1);
    const scope: Element = modal ?? document.body;
    const vw = document.documentElement.clientWidth, vh = document.documentElement.clientHeight;
    const seen = new Set<string>();
    let n = 0;
    const ownText = (el: Element) => [...el.childNodes].some((c) => c.nodeType === Node.TEXT_NODE && (c.textContent ?? '').trim());
    for (const el of scope.querySelectorAll<HTMLElement>(selector)) {
      if (el.closest('[aria-hidden="true"], [inert]')) continue;
      if (!el.checkVisibility({ opacityProperty: true, visibilityProperty: true })) continue;
      const box = el.getBoundingClientRect();
      if (!box.width || !box.height || box.bottom <= 0 || box.right <= 0 || box.top >= vh || box.left >= vw) continue;
      // A control scrolled under a sticky head (or any other panel) is not on screen: its pixels are the cover's.
      // Its middle and both ends of its height: a row half under a pinned footer is not on screen either.
      const covered = [0.5, 0.1, 0.9].some((f) => {
        const hit = document.elementFromPoint(Math.min(vw - 1, Math.max(0, box.left + box.width / 2)), Math.min(vh - 1, Math.max(0, box.top + box.height * f)));
        return !!hit && !el.contains(hit);
      });
      if (covered) continue;
      // Screen-reader-only text (a 1 px clipped box, e.g. a cost chip's sentence) is never seen.
      const shown = (t: HTMLElement) => { const b = t.getBoundingClientRect(); return b.width > 1 && b.height > 1; };
      const runs = [el, ...el.querySelectorAll<HTMLElement>('*')].filter((t) => ownText(t) && !t.closest('[aria-hidden="true"]') && t.checkVisibility({ opacityProperty: true, visibilityProperty: true }) && shown(t));
      if (!runs.length) continue;
      const states = ['aria-pressed', 'aria-selected', 'aria-checked', 'aria-expanded', 'aria-current'].map((a) => `${a}=${el.getAttribute(a) ?? ''}`);
      // The same classes can look different in another container (a .seg inside the game menu).
      const container: string[] = [];
      for (let p = el.parentElement; p && container.length < 3; p = p.parentElement) if (p.className) container.push(String(p.className));
      const key = `${el.tagName}|${el.className}|${el.getAttribute('role')}|${el.matches(':disabled')}|${states.join(',')}|${container}|${runs.map((t) => t.className).join('/')}`;
      if (seen.has(key)) continue;
      seen.add(key);
      el.setAttribute('data-cx', String(n++));
      for (const t of runs) t.setAttribute('data-cx-run', '');
    }
    return n;
  }, CONTROLS);
}

type Box = { x: number; y: number; w: number; h: number };

async function measureAll(page: Page, state: string): Promise<ContrastReading[]> {
  // Where each label's first line of text is, and what it is.
  const runs = await page.evaluate((state) => {
    const vw = document.documentElement.clientWidth, vh = document.documentElement.clientHeight;
    const out: { box: Box; large: boolean; state: string; where: string; text: string }[] = [];
    const cls = (e: Element) => (typeof e.className === 'string' && e.className.trim() ? `.${e.className.trim().split(/\s+/).join('.')}` : '');
    for (const t of document.querySelectorAll<HTMLElement>('[data-cx-run]')) {
      const el = t.closest<HTMLElement>('[data-cx]')!;
      const disabled = el.matches(':disabled, [aria-disabled="true"]');
      // A disabled control has no interaction states: it is measured once, as "disabled".
      if (disabled && state !== 'default') continue;
      const range = document.createRange();
      let r: DOMRect | undefined;
      for (const c of t.childNodes) {
        if (r || c.nodeType !== Node.TEXT_NODE || !(c.textContent ?? '').trim()) continue;
        range.selectNodeContents(c);
        r = [...range.getClientRects()].find((b) => b.width > 2 && b.height > 2);
      }
      if (!r) continue;
      const x = Math.max(0, Math.floor(r.left)), y = Math.max(0, Math.floor(r.top));
      const w = Math.min(vw, Math.ceil(r.right)) - x, h = Math.min(vh, Math.ceil(r.bottom)) - y;
      if (w < 3 || h < 3) continue;
      // Nothing on top of the text (a tooltip or popover): measure only what the player sees.
      const hit = document.elementFromPoint(x + w / 2, y + h / 2);
      if (!hit || !el.contains(hit)) continue;
      const s = getComputedStyle(t);
      const size = parseFloat(s.fontSize), weight = Number(s.fontWeight) || 400;
      const label = (el.getAttribute('aria-label') || el.innerText || '').trim().replace(/\s+/g, ' ').slice(0, 36);
      out.push({
        box: { x, y, w, h },
        large: size >= 24 || (size >= 18.66 && weight >= 700),
        state: disabled ? 'disabled' : state,
        where: `${el.tagName.toLowerCase()}${cls(el)} "${label}"`,
        text: t === el ? 'label' : `${t.tagName.toLowerCase()}${cls(t)}`,
      });
    }
    return out;
  }, state);
  if (!runs.length) return [];
  const setClass = (name: string | null) => page.evaluate((name) => {
    for (const t of document.querySelectorAll('[data-cx-run]')) {
      t.classList.remove('cx-thick', 'cx-hide');
      if (name) t.classList.add(name);
    }
  }, name);
  await setClass('cx-thick');
  const ink = (await page.screenshot({ animations: 'disabled', caret: 'hide' })).toString('base64');
  await setClass('cx-hide');
  const paper = (await page.screenshot({ animations: 'disabled', caret: 'hide' })).toString('base64');
  await setClass(null);
  const results = await page.evaluate(async ({ ink, paper, boxes }) => {
    const decode = async (b64: string) => {
      const blob = await (await fetch(`data:image/png;base64,${b64}`)).blob();
      const bmp = await createImageBitmap(blob);
      const canvas = new OffscreenCanvas(bmp.width, bmp.height);
      const ctx = canvas.getContext('2d')!;
      ctx.drawImage(bmp, 0, 0);
      return { data: ctx.getImageData(0, 0, bmp.width, bmp.height).data, width: bmp.width, scale: bmp.width / document.documentElement.clientWidth };
    };
    const a = await decode(ink), b = await decode(paper);
    const lum = (r: number, g: number, bl: number) => {
      const f = (v: number) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; };
      return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(bl);
    };
    const ratio = (l1: number, l2: number) => (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
    const hex = (c: number[]) => `#${c.slice(0, 3).map((v) => v.toString(16).padStart(2, '0')).join('')}`;
    return boxes.map(({ x, y, w, h }) => {
      const k = a.scale;
      const px: number[] = [];
      for (let yy = Math.floor(y * k); yy < Math.floor((y + h) * k); yy++) {
        for (let xx = Math.floor(x * k); xx < Math.floor((x + w) * k); xx++) px.push((yy * a.width + xx) * 4);
      }
      // Background: the median-luminance pixel of the text-free shot.
      const bgs = px.map((i) => [b.data[i], b.data[i + 1], b.data[i + 2], lum(b.data[i], b.data[i + 1], b.data[i + 2])]).sort((p, q) => p[3] - q[3]);
      const bg = bgs[Math.floor(bgs.length / 2)];
      // Ink: of the pixels the text changed, the one furthest from the background (a solid core).
      let best = 1, fg = bg;
      for (const i of px) {
        if (Math.abs(a.data[i] - b.data[i]) + Math.abs(a.data[i + 1] - b.data[i + 1]) + Math.abs(a.data[i + 2] - b.data[i + 2]) < 6) continue;
        const c = ratio(lum(a.data[i], a.data[i + 1], a.data[i + 2]), bg[3]);
        if (c > best) { best = c; fg = [a.data[i], a.data[i + 1], a.data[i + 2]]; }
      }
      return { ratio: Math.round(best * 100) / 100, fg: hex(fg), bg: hex(bg) };
    });
  }, { ink, paper, boxes: runs.map((r) => r.box) });
  return runs.map((r, i) => ({ where: r.where, state: r.state, text: r.text, ratio: results[i].ratio, required: r.large ? 3 : 4.5, fg: results[i].fg, bg: results[i].bg }));
}
