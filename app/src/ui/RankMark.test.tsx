import { describe, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { RankMark } from './RankMark';

describe('unit rank marks', () => {
  it.each([1, 2, 3, 4, 5] as const)('level %i uses a gold enlisted badge', (level) => {
    const html = renderToStaticMarkup(<RankMark faction="India" level={level} />);
    expect(html).toContain('class="rank-insignia"');
    expect(html).toContain(`aria-label="${level === 5 ? 'Veteran, level 5' : `Level ${level}`}"`);
    expect(html.match(/<g>/g)).toHaveLength(level);
  });

  it('Sparta level 6 keeps the gold family and adds its centre Lambda', () => {
    const html = renderToStaticMarkup(<RankMark faction="Sparta" level={6} />);
    expect(html).toContain('aria-label="Spartan Veteran, level 6"');
    expect(html.match(/<g>/g)).toHaveLength(6);
    expect(html).toContain('M9 21 L12 16 L15 21');
  });

  it('shows nothing before the first promotion', () => {
    expect(renderToStaticMarkup(<RankMark faction="India" level={0} />)).toBe('');
  });
});
