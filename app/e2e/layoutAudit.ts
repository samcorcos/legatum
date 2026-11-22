import type { Page } from '@playwright/test';

/** One layout problem found on the current screen. */
export interface LayoutIssue {
  kind: 'offscreen' | 'clipped' | 'h-overflow' | 'overlap' | 'text-clipped' | 'text-overflow' | 'small-target';
  /** A short CSS-ish path to the element, plus its accessible text. */
  where: string;
  detail: string;
}

export interface AuditOptions {
  /** Smallest acceptable pointer target (px, both sides). Desktop: 32. */
  minTarget?: number;
}

/** Named reservations are disjoint, including the actual phone canvas. Inspector content can
 * be a fixed child of a small trigger slot, so compare geometry even for DOM ancestors. */
export async function auditOverlaySlots(page: Page): Promise<LayoutIssue[]> {
  return page.evaluate(() => {
    const slots = [...document.querySelectorAll<HTMLElement>('[data-overlay-slot]')]
      .filter((el) => !el.closest('[aria-hidden="true"], [inert]') && el.checkVisibility({ opacityProperty: true, visibilityProperty: true }))
      .map((el) => ({ name: el.dataset.overlaySlot!, b: el.getBoundingClientRect() }))
      .filter(({ b }) => b.width > 1 && b.height > 1);
    const issues: LayoutIssue[] = [];
    for (let i = 0; i < slots.length; i++) {
      const { name, b } = slots[i];
      if (b.left < -1 || b.top < -1 || b.right > innerWidth + 1 || b.bottom > innerHeight + 1) {
        issues.push({ kind: 'offscreen', where: name, detail: `${b.x},${b.y} ${b.width}x${b.height}` });
      }
      for (const other of slots.slice(i + 1)) {
        const w = Math.min(b.right, other.b.right) - Math.max(b.left, other.b.left);
        const h = Math.min(b.bottom, other.b.bottom) - Math.max(b.top, other.b.top);
        if (w > 1 && h > 1) issues.push({ kind: 'overlap', where: name, detail: `${other.name}: ${w}x${h}` });
      }
    }
    return issues;
  });
}

/**
 * Geometry audit of what is on screen now. It checks only meaningful elements (controls and
 * text), skips decoration (aria-hidden, pointer-events: none art) and, when a modal dialog is
 * open, looks only inside the top-most one (the rest of the page is inert under it).
 *
 * - offscreen: a control or text is (partly) outside the viewport and no scroll container can
 *   bring it back.
 * - clipped: a control or text is cut off by an ancestor that hides overflow and cannot scroll.
 * - h-overflow: the page itself scrolls sideways.
 * - overlap: two visible controls intersect.
 * - text-clipped: a text element hides overflowing text without an ellipsis or a scrollbar.
 * - text-overflow: text spills sideways out of its own box (into neighbours, or cut further up).
 * - small-target: a control smaller than `minTarget` on either side (inline text links exempt).
 */
export async function auditLayout(page: Page, options: AuditOptions = {}): Promise<LayoutIssue[]> {
  return page.evaluate(({ minTarget }) => {
    const issues: { kind: string; where: string; detail: string }[] = [];
    const vw = document.documentElement.clientWidth;
    const vh = document.documentElement.clientHeight;
    const TOL = 1.5;

    const describe = (el: Element) => {
      const parts: string[] = [];
      let node: Element | null = el;
      for (let i = 0; node && i < 3; i++, node = node.parentElement) {
        const cls = [...node.classList].slice(0, 2).join('.');
        parts.unshift(node.tagName.toLowerCase() + (cls ? `.${cls}` : ''));
      }
      const label = (el.getAttribute('aria-label') || (el as HTMLElement).innerText || '').trim().replace(/\s+/g, ' ').slice(0, 40);
      return `${parts.join(' > ')}${label ? ` "${label}"` : ''}`;
    };
    const r = (n: number) => Math.round(n);
    const box = (b: DOMRect) => `[${r(b.left)},${r(b.top)} ${r(b.width)}x${r(b.height)}]`;

    const modals = [...document.querySelectorAll('dialog')].filter((d) => d.matches(':modal'));
    const scope: Element = modals.at(-1) ?? document.body;

    const visible = (el: Element) => {
      // Screen-reader-only text is clipped to 1px by design (amounts carry their word this way).
      if (el.closest('[aria-hidden="true"], [inert], .sr-only')) return false;
      const s = getComputedStyle(el);
      if (s.visibility !== 'visible' || s.display === 'contents') return false;
      // Walk up for opacity 0 or display none (checkVisibility covers display).
      if (!el.checkVisibility({ opacityProperty: true, visibilityProperty: true })) return false;
      const b = el.getBoundingClientRect();
      return b.width > 0 && b.height > 0;
    };
    const CONTROL = 'button, a[href], input:not([type="hidden"]), select, textarea, summary, [role="button"], [role="radio"], [role="tab"], [role="checkbox"]';
    const hasOwnText = (el: Element) => [...el.childNodes].some((n) => n.nodeType === Node.TEXT_NODE && (n.textContent ?? '').trim().length > 0);

    // Clip chain: every ancestor that clips overflow, up to (not including) the scope root's parent.
    const clippers = (el: Element) => {
      const out: { el: Element; scrollable: boolean }[] = [];
      for (let p = el.parentElement; p && p !== document.documentElement; p = p.parentElement) {
        const s = getComputedStyle(p);
        const clipX = s.overflowX !== 'visible', clipY = s.overflowY !== 'visible';
        if (clipX || clipY) {
          const scrollable = /(auto|scroll)/.test(s.overflowX + s.overflowY);
          out.push({ el: p, scrollable });
        }
        if (s.position === 'fixed') break;
      }
      return out;
    };

    const all = [...scope.querySelectorAll('*')].filter((el) => !(el instanceof SVGElement && !(el instanceof SVGSVGElement)));
    const meaningful = all.filter((el) => (el.matches(CONTROL) || hasOwnText(el)) && visible(el));

    for (const el of meaningful) {
      const b = el.getBoundingClientRect();
      const chain = clippers(el);
      // Inside a scroll container: it's reachable if it's within the container's scroll extent,
      // and the container itself must be on screen (checked when we reach it as meaningful? no:
      // containers may hold no text). Check the nearest scroller's rect instead of the element.
      const scroller = chain.find((c) => c.scrollable);
      const hardClips = chain.filter((c) => !c.scrollable && (!scroller || chain.indexOf(c) < chain.indexOf(scroller)));
      for (const c of hardClips) {
        const cb = c.el.getBoundingClientRect();
        const s = getComputedStyle(c.el);
        // Relation badges have a transparent 5px hit target outside their painted inset frame.
        // Judge the visible frame, while still catching a genuinely clipped badge label.
        const badge = el.matches('.relation-badge') ? getComputedStyle(el) : null;
        const top = b.top + (badge?.borderTopColor === 'rgba(0, 0, 0, 0)' ? parseFloat(badge.borderTopWidth) : 0);
        const bottom = b.bottom - (badge?.borderBottomColor === 'rgba(0, 0, 0, 0)' ? parseFloat(badge.borderBottomWidth) : 0);
        const cutX = s.overflowX !== 'visible' && (b.left < cb.left - TOL || b.right > cb.right + TOL);
        const cutY = s.overflowY !== 'visible' && (top < cb.top - TOL || bottom > cb.bottom + TOL);
        // Entirely hidden elements (e.g. collapsed accordions using overflow) are not "cut".
        const inside = b.right > cb.left && b.left < cb.right && b.bottom > cb.top && b.top < cb.bottom;
        if ((cutX || cutY) && inside) {
          issues.push({ kind: 'clipped', where: describe(el), detail: `${box(b)} cut by ${describe(c.el)} ${box(cb)}` });
          break;
        }
      }
      // Nested scrollers: the outermost one can bring the inner ones into view, so it is the one
      // that has to be on screen.
      const outer = chain.filter((c) => c.scrollable).at(-1);
      const target = outer ? outer.el.getBoundingClientRect() : b;
      const ref = outer ? outer.el : el;
      // Only elements that are at least partly visible matter for offscreen (fully scrolled-away
      // content inside a scroller is reachable).
      if (target.left < -TOL || target.top < -TOL || target.right > vw + TOL || target.bottom > vh + TOL) {
        issues.push({ kind: 'offscreen', where: describe(ref), detail: `${box(target)} outside ${vw}x${vh}` });
      }
      if (hasOwnText(el)) {
        const s = getComputedStyle(el);
        const hides = s.overflowX === 'hidden' || s.overflowY === 'hidden' || s.overflowX === 'clip' || s.overflowY === 'clip';
        const e = el as HTMLElement;
        const over = e.scrollHeight > e.clientHeight + 1 || e.scrollWidth > e.clientWidth + 1;
        if (hides && over && s.textOverflow !== 'ellipsis' && !(s.webkitLineClamp && s.webkitLineClamp !== 'none')) {
          issues.push({ kind: 'text-clipped', where: describe(el), detail: `scroll ${e.scrollWidth}x${e.scrollHeight} > client ${e.clientWidth}x${e.clientHeight}` });
        } else if (!hides && s.display !== 'inline' && e.clientWidth > 0 && e.scrollWidth > e.clientWidth + 1 && !/(auto|scroll)/.test(s.overflowX)) {
          // Text spilling sideways out of its own box: it runs into neighbours or gets cut further up.
          issues.push({ kind: 'text-overflow', where: describe(el), detail: `text ${e.scrollWidth}px wide in a ${e.clientWidth}px box` });
        }
      }
    }

    if (document.documentElement.scrollWidth > document.documentElement.clientWidth) {
      issues.push({ kind: 'h-overflow', where: 'html', detail: `scrollWidth ${document.documentElement.scrollWidth} > ${document.documentElement.clientWidth}` });
    }

    // Controls that are actually hit-testable somewhere (not fully covered or scrolled away).
    const controls = meaningful.filter((el) => el.matches(CONTROL)).filter((el) => {
      const b = el.getBoundingClientRect();
      const pts = [[b.left + b.width / 2, b.top + b.height / 2], [b.left + 3, b.top + 3], [b.right - 3, b.bottom - 3]];
      return pts.some(([x, y]) => {
        const hit = document.elementFromPoint(x, y);
        return !!hit && (hit === el || el.contains(hit) || hit.contains(el) || !!hit.closest('label')?.contains(el));
      });
    });
    // The part of a control that can actually be seen: its box cut by every clipping ancestor.
    const visibleRect = (el: Element) => {
      const v = el.getBoundingClientRect();
      let { left, top, right, bottom } = v;
      for (const c of clippers(el)) {
        const cb = c.el.getBoundingClientRect();
        const s = getComputedStyle(c.el);
        if (s.overflowX !== 'visible') { left = Math.max(left, cb.left); right = Math.min(right, cb.right); }
        if (s.overflowY !== 'visible') { top = Math.max(top, cb.top); bottom = Math.min(bottom, cb.bottom); }
      }
      return { left, top, right, bottom };
    };
    for (let i = 0; i < controls.length; i++) {
      for (let j = i + 1; j < controls.length; j++) {
        const a = controls[i], c = controls[j];
        if (a.contains(c) || c.contains(a)) continue;
        const ab = visibleRect(a), cb = visibleRect(c);
        const w = Math.min(ab.right, cb.right) - Math.max(ab.left, cb.left);
        const h = Math.min(ab.bottom, cb.bottom) - Math.max(ab.top, cb.top);
        if (w > 2 && h > 2) issues.push({ kind: 'overlap', where: describe(a), detail: `intersects ${describe(c)} by ${r(w)}x${r(h)}` });
      }
    }

    for (const el of controls) {
      if (el.matches('a') && el.parentElement && getComputedStyle(el).display === 'inline') continue;
      // A checkbox or radio inside its label: the label is the target.
      const hit = el.matches('input[type="checkbox"], input[type="radio"]') ? el.closest('label') ?? el : el;
      const b = hit.getBoundingClientRect();
      if (b.width < minTarget - 0.5 || b.height < minTarget - 0.5) {
        issues.push({ kind: 'small-target', where: describe(hit), detail: `${r(b.width)}x${r(b.height)} < ${minTarget}` });
      }
    }
    return issues;
  }, { minTarget: options.minTarget ?? 32 }) as Promise<LayoutIssue[]>;
}

/**
 * The visible text of the current screen as blocks, cut the way a reader reads: each text node joins the
 * text of its nearest block-level element (by tag, like `htmlBlocks` in `src/test/textArtifacts.ts`, never
 * by computed display: an Amount is an inline-flex whose number and word are flex items, yet it reads as
 * one run). Hidden and aria-hidden decoration (list dots, icon masks) is skipped. Feeds the text-artifact guard.
 */
export async function visibleTextBlocks(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const BLOCK = 'p, div, li, ul, ol, dl, dt, dd, h1, h2, h3, h4, h5, h6, tr, td, th, table, section, article, header, footer, button, summary, details, dialog, nav, aside, figure, figcaption, select, option, label, form, blockquote';
    const blocks = new Map<Element, string>();
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      const text = node.textContent ?? '';
      const parent = node.parentElement;
      if (!text.trim() || !parent || parent.closest('script, style, noscript, [aria-hidden="true"], [inert]')) continue;
      if (!parent.checkVisibility({ visibilityProperty: true })) continue;
      const block = parent.closest(BLOCK) ?? document.body;
      blocks.set(block, (blocks.get(block) ?? '') + text);
    }
    return [...blocks.values()].map((b) => b.trim()).filter(Boolean);
  });
}
