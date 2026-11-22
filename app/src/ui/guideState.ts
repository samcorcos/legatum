import { create } from 'zustand';

export type GuideSection = 'start' | 'turns' | 'cities' | 'combat' | 'victory' | 'factions' | 'controls' | 'credits';

interface GuideState {
  open: boolean;
  section: GuideSection;
  show(section?: GuideSection): void;
  close(): void;
}

export const useGuide = create<GuideState>((set) => ({
  open: false,
  section: 'start',
  show: (section = 'start') => set({ open: true, section }),
  close: () => set({ open: false }),
}));

export const openGuide = () => useGuide.getState().show();

/** A question mark typed into a name, seed, or other editable field is still text. */
export function isGuideShortcut(e: KeyboardEvent): boolean {
  const target = e.target as HTMLElement | null;
  return e.key === '?' && !e.repeat && !e.ctrlKey && !e.metaKey && !e.altKey &&
    !target?.closest?.('input, textarea, select, [contenteditable]:not([contenteditable="false"])');
}
