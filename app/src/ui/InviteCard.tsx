import { useEffect, useRef, useState } from 'react';
import { listedRoomName } from '../net/roomName';
import { BUILD_ID, buildUrl, newestUrl } from '../platform/versions';

/** The link that opens this build's Multiplayer and joins the room: the same version, so a friend never lands on a newer one. */
export function inviteLink(room: string): string {
  return BUILD_ID ? buildUrl(BUILD_ID, { join: room }) : newestUrl({ join: room });
}

type Copy = 'idle' | 'copied' | 'failed';

/**
 * The host's invitation, a band under the game room's title: the game's name as the game list shows it, the private code
 * when there is one, and a button that copies a link that joins this game. The link never carries the code.
 */
export function InviteCard({ room, code }: { room: string; code: string | null }) {
  const [copy, setCopy] = useState<Copy>('idle');
  const link = inviteLink(room);
  const shown = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (copy !== 'copied') return;
    const t = setTimeout(() => setCopy('idle'), 4000);
    return () => clearTimeout(t);
  }, [copy]);
  useEffect(() => {
    if (copy === 'failed') shown.current?.select();
  }, [copy]);
  const copyLink = async () => {
    try {
      await navigator.clipboard.writeText(link);
      setCopy('copied');
    } catch {
      setCopy('failed');
    }
  };
  return (
    <section className="invite-card" aria-labelledby="invite-name">
      <div className="invite-id">
        <div className="invite-kicker">Invite friends</div>
        <div id="invite-name" className="invite-name">{listedRoomName(room)}</div>
      </div>
      <div className="invite-how">
        <p>Friends find this name under Multiplayer, Join. Or send them the link.</p>
        {code && <p className="invite-row">Private code <b className="invite-code">{code}</b></p>}
      </div>
      <div className="invite-act">
        <button className="btn invite-copy" onClick={copyLink}>
          {copy === 'copied' ? 'Link copied' : 'Copy invite link'}
        </button>
        {copy === 'failed' ? (
          <label className="invite-fallback">
            <span>Copy this link:</span>
            <input ref={shown} className="lobby-input" readOnly value={link} onFocus={(e) => e.currentTarget.select()} />
          </label>
        ) : (
          <p className="invite-status" role="status">
            {copy === 'copied' ? (code ? 'Send it with the private code.' : 'Send it to your friends.') : ''}
          </p>
        )}
      </div>
    </section>
  );
}
