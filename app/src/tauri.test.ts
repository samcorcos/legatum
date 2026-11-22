import { describe, expect, it } from 'vitest';
import { isMobileDevice } from './tauri';

const MAC = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)';
const IPHONE = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148';
const WINDOWS = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0 Safari/537.36';

describe('isMobileDevice', () => {
  it('treats a Mac without touch as desktop', () => expect(isMobileDevice(MAC, 0)).toBe(false));
  // The iPad WKWebView (seen in the Tauri iOS app on the simulator) sends this Mac user agent.
  it('treats a Mac user agent with touch points as iPad', () => expect(isMobileDevice(MAC, 5)).toBe(true));
  it('detects iPhone', () => expect(isMobileDevice(IPHONE, 5)).toBe(true));
  it('keeps a Windows touch laptop on desktop', () => expect(isMobileDevice(WINDOWS, 10)).toBe(false));
});
