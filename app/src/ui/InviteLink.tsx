import { useState } from 'react';

/** "Players on your network can join at http://<lan-ip>:7878", with a copy button. */
export function InviteLink({ urls, compact }: { urls: string[]; compact?: boolean }) {
  const [copied, setCopied] = useState(false);
  const url = urls[0];
  if (!url) return null;
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard blocked: the URL is visible anyway.
    }
  };
  if (compact) {
    return (
      <button className="invite-chip" onClick={copy} title="Copy the join link for players on your network">
        <span className="invite-dot" /><span className="touch-only">Copy join link </span>
        {copied ? 'Copied' : url.replace(/^https?:\/\//, '')}
      </button>
    );
  }
  return (
    <div className="invite">
      <div className="lbl">Players on your network can join at</div>
      <div className="invite-row">
        <code className="invite-url">{url}</code>
        <button className="btn" onClick={copy}>
          {copied ? 'Copied' : 'Copy'}
        </button>
      </div>
      {urls.length > 1 && <div className="muted invite-more">Also: {urls.slice(1).join(' · ')}</div>}
      <div className="muted invite-more">Same Wi-Fi or network; nothing to install.</div>
    </div>
  );
}
