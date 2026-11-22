import { useGame } from '../store';
import { typedJoinCode, JOIN_CODE_MAX } from '../net/joinCode';

/** A guest's box for the host's join code on a locked game. Typed once; the seat buttons wait for it. */
export function JoinCodeField() {
  const text = useGame((s) => s.joinCodeText);
  const setText = useGame((s) => s.setJoinCodeText);
  return (
    <div className="join-code">
      <label className="join-code-label" htmlFor="join-code-input">Join code</label>
      <input
        id="join-code-input"
        className="lobby-input"
        value={text}
        maxLength={JOIN_CODE_MAX}
        autoComplete="off"
        autoCapitalize="off"
        spellCheck={false}
        aria-describedby="join-code-hint"
        onChange={(e) => setText(typedJoinCode(e.target.value))}
      />
      <span id="join-code-hint" className="join-code-hint">
        {text ? 'Now pick a seat.' : 'Type the code the host gave you, then pick a seat.'}
      </span>
    </div>
  );
}
