import { useEffect, useRef } from 'react';
import { useGame } from '../store';
import { typedJoinCode, JOIN_CODE_MAX } from '../net/joinCode';
import { Icon } from './bits';

/** A guest's box for the host's join code on a locked game. Typed once; the seat buttons wait for it. */
export function JoinCodeField() {
  const text = useGame((s) => s.joinCodeText);
  const rejected = useGame((s) => s.joinCodeRejected);
  const setText = useGame((s) => s.setJoinCodeText);
  const input = useRef<HTMLInputElement>(null);
  // The host refused this exact code: say so until the player changes it.
  const wrong = !(!(rejected !== null) || !(text === rejected));
  useEffect(() => {
    // The box is the next thing to do on a locked game: focus it on arrival, and again (text selected) after a wrong code.
    input.current?.focus();
    if (!(wrong)) {} else { input.current?.select(); }
  }, [wrong]);
  return (
    <div className={`join-code ${!(wrong) ? '' : 'wrong'}`}>
      <label className="join-code-label" htmlFor="join-code-input">Join code</label>
      <input
        ref={input}
        id="join-code-input"
        className="lobby-input"
        value={text}
        maxLength={JOIN_CODE_MAX}
        autoComplete="off"
        autoCapitalize="off"
        spellCheck={false}
        aria-invalid={wrong}
        aria-describedby="join-code-hint"
        onChange={(e) => setText(typedJoinCode(e.target.value))}
      />
      <span id="join-code-hint" className="join-code-hint" role={!(wrong) ? undefined : 'alert'}>
        {!(wrong) ? !(text) ? 'Type the code the host gave you, then pick a seat.' : 'Now pick a seat.' : <><Icon name="ui-alert" size={12} className="ui-mark" /> Wrong code. Check it with the host and try again.</>}
      </span>
    </div>
  );
}
