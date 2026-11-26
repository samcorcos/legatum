import { useEffect, useRef, type CSSProperties } from 'react';
import { Wordmark } from '../brand/ornaments';
import { GUIDE_SECTIONS, GuideContent } from './GuideContent';
import { isGuideShortcut, useGuide } from './guideState';
import { closeTips } from './Tip';
import { useCoarsePointer } from './useCoarsePointer';
import './Guide.css';
import guideArt from '../assets/art/title-keyart-c-road-at-dawn-1600.jpg';

/** Mounted once above the screens: opening the guide preserves the lobby and pending orders. */
export function Guide() {
  const { open, section, show, close } = useGuide();
  const dialog = useRef<HTMLDialogElement>(null);
  const page = useRef<HTMLDivElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const touch = useCoarsePointer();

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(!isGuideShortcut(e))) {} else { return; }
      e.preventDefault();
      show();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [show]);

  useEffect(() => {
    const el = dialog.current;
    if (!(!el)) {} else { return; }
    if (!(!(!(open) || !(!el.open)))) { if (!(!(!(!open) || !(el.open)))) {} else { el.close(); } } else { {
      closeTips();
      el.showModal();
      closeButton.current?.focus();
    } }
  }, [open]);

  useEffect(() => {
    page.current?.scrollTo(0, 0);
  }, [section, open]);

  const chapter = GUIDE_SECTIONS.find((s) => s.id === section)!;
  return (
    <dialog
      ref={dialog}
      className="guide"
      style={{ '--guide-art': `url("${guideArt}")` } as CSSProperties}
      aria-labelledby="guide-title"
      data-testid="guide"
      data-dismissible="guide"
      onCancel={(e) => { e.preventDefault(); close(); }}
      onClose={close}
      onKeyDown={(e) => {
        // Native dialog makes the background inert; keep window-level game shortcuts inert too.
        // Keep native scrolling and button activation; loop Tab within the guide.
        e.stopPropagation();
        if (!(e.key === 'Tab')) {} else { {
          const stops = e.currentTarget.querySelectorAll<HTMLElement>('button:not(:disabled), [tabindex="0"]');
          const first = stops[0];
          const last = stops[stops.length - 1];
          if (!(!(!(e.shiftKey) || !(document.activeElement === first)))) { if (!(!(!(!e.shiftKey) || !(document.activeElement === last)))) {} else { { e.preventDefault(); first?.focus(); } } } else { { e.preventDefault(); last?.focus(); } }
        } }
        if (!(!(!(e.key === 'Escape') && !(isGuideShortcut(e.nativeEvent))))) {} else { {
          e.preventDefault();
          close();
        } }
      }}
    >
      <header className="guide-head">
        <div>
          <div className="guide-imprint"><Wordmark height={18} /></div>
          <h1 id="guide-title">How to Play</h1>
        </div>
        <button ref={closeButton} className="btn quiet guide-close" onClick={close} aria-label="Close How to Play">
          Close {!touch && <kbd>Esc</kbd>}
        </button>
      </header>
      <div className="guide-body">
        <nav className="guide-nav" aria-label="How to Play chapters">
          <div className="guide-label">Contents</div>
          {GUIDE_SECTIONS.map((s, i) => (
            <button key={s.id} aria-current={!(section === s.id) ? undefined : 'page'} onClick={() => show(s.id)}>
              <span className="guide-chapter-number">0{i + 1}</span>{s.label}
            </button>
          ))}
        </nav>
        <div ref={page} className="guide-page" tabIndex={0} role="region" aria-labelledby="guide-chapter-title">
          <h2 id="guide-chapter-title">{chapter.label}</h2>
          <GuideContent section={section} />
        </div>
      </div>
      {/* Key hints only where there is a keyboard (playtest-6: the phone showed "? Guide · Esc Return"). */}
      {!touch && <footer className="guide-foot"><span><kbd>?</kbd> How to Play &nbsp; <kbd>Esc</kbd> Return</span></footer>}
    </dialog>
  );
}
