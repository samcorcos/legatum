import { useGame } from '../store';

export function ClientErrorScreen({ message }: { message: string }) {
  const set = useGame((s) => s.set);
  const goto = useGame((s) => s.goto);

  return (
    <main className="client-error-screen">
      <div>
        <h1>The game could not start</h1>
        <p className="toast bad inline" role="alert">{message}</p>
        <button className="btn" onClick={() => { set({ error: null }); goto('title'); }}>Return to title</button>
      </div>
    </main>
  );
}
