import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ActivityView } from './ActivityView';
import { ApprovalPanel } from './ApprovalPanel';
import { useBridge } from './bridge';
import { Dialog } from './Dialog';
import { BloomMark, Icon } from './Icons';
import { PieMenu } from './PieMenu';
import { shortcutLabel, useNativeShortcut } from './native';

type Preferences = {
  accent: 'sage' | 'lilac';
  rotation: number;
  reducedMotion: boolean;
  shortcut: boolean;
};
const defaults: Preferences = {
  accent: 'sage',
  rotation: -90,
  reducedMotion: false,
  shortcut: true,
};
function savedPreferences(): Preferences {
  try {
    const saved = JSON.parse(localStorage.getItem('bloom.preferences') ?? '{}');
    return {
      accent: saved.accent === 'lilac' ? 'lilac' : 'sage',
      rotation: saved.rotation === 0 ? 0 : -90,
      reducedMotion: saved.reducedMotion === true,
      shortcut: saved.shortcut !== false,
    };
  } catch {
    return defaults;
  }
}

export function App() {
  const { state, connection, error, dismissError, send } = useBridge();
  const [page, setPage] = useState<'activity' | 'pie' | 'settings'>('activity');
  const [prefs, setPrefs] = useState(savedPreferences);
  const [submitted, setSubmitted] = useState(false);
  const [nativePie, setNativePie] = useState(isTauri());
  const [nativeError, setNativeError] = useState<string | null>(null);
  const [previewSelection, setPreviewSelection] = useState('');
  const shownAt = useRef<string | null>(null);
  const submitting = useRef(false);
  const starting = useRef(false);
  const previousPhase = useRef(state.phase);
  const pieWindow = new URLSearchParams(location.search).get('view') === 'pie';
  const busy = state.phase !== 'idle';
  const connected = connection === 'connected';
  const plan = state.plan;
  const latestPlan = plan ?? state.history.find((run) => run.plan)?.plan;
  const phaseId = `${state.phase}:${plan?.request_id ?? ''}:${state.approval?.run_id ?? ''}`;

  useEffect(() => {
    starting.current = false;
    submitting.current = false;
    shownAt.current = null;
    setSubmitted(false);
  }, [phaseId]);
  useEffect(() => {
    if (error) {
      starting.current = false;
      submitting.current = false;
      setSubmitted(false);
    }
  }, [error]);
  useEffect(() => {
    localStorage.setItem('bloom.preferences', JSON.stringify(prefs));
    document.documentElement.dataset.accent = prefs.accent;
    document.documentElement.classList.toggle('reduce-motion', prefs.reducedMotion);
  }, [prefs]);

  useEffect(() => {
    const syncPreferences = (event: StorageEvent) => {
      if (event.key === 'bloom.preferences') setPrefs(savedPreferences());
    };
    window.addEventListener('storage', syncPreferences);
    return () => window.removeEventListener('storage', syncPreferences);
  }, []);

  const start = useCallback(
    (prepared = false) => {
      if (!connected || busy || starting.current) return;
      starting.current = true;
      const launch = () => {
        if (!send({ type: 'start' })) starting.current = false;
      };
      if (isTauri() && !prepared) invoke('prepare_pie').then(launch).catch(launch);
      else launch();
    },
    [connected, busy, send],
  );
  const shortcutStatus = useNativeShortcut(start, prefs.shortcut && connected, !pieWindow);
  const onShown = useCallback((at: string) => {
    shownAt.current = at;
  }, []);
  const choose = useCallback(
    (actionId: string | null) => {
      if (!plan || submitting.current) return;
      const sent = send({
        type: 'choice',
        request_id: plan.request_id,
        choice: {
          action_id: actionId,
          shown_at: shownAt.current ?? new Date().toISOString(),
        },
      });
      submitting.current = sent;
      setSubmitted(sent);
    },
    [plan, send],
  );
  const respond = useCallback(
    (verdict: 'approve' | 'reject') => {
      if (!state.approval || submitting.current) return;
      const sent = send({
        type: 'approval',
        response: { run_id: state.approval.run_id, verdict, edited_args: null },
      });
      submitting.current = sent;
      setSubmitted(sent);
    },
    [state.approval, send],
  );

  useEffect(() => {
    if (!isTauri() || !pieWindow) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    listen('bloom-cancel', () => choose(null))
      .then((remove) => {
        if (disposed) remove();
        else unlisten = remove;
      })
      .catch(() => {});
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [choose, pieWindow]);

  useEffect(() => {
    function shortcut(event: KeyboardEvent) {
      if (
        prefs.shortcut &&
        !shortcutStatus.registered &&
        (event.ctrlKey || event.metaKey) &&
        event.code === 'Space'
      ) {
        event.preventDefault();
        if (!event.repeat) start();
      }
    }
    window.addEventListener('keydown', shortcut);
    return () => window.removeEventListener('keydown', shortcut);
  }, [start, prefs.shortcut, shortcutStatus.registered]);

  useEffect(() => {
    if (!isTauri() || pieWindow) return;
    if (state.phase === 'choice') {
      invoke<boolean>('show_pie')
        .then((anchored) => {
          if (!anchored)
            setNativeError(
              'Your desktop controls overlay placement because global cursor positioning is unavailable.',
            );
        })
        .catch(() => {
          setNativePie(false);
          setNativeError(
            'The floating window could not open. The pie menu is available in this window.',
          );
        });
    } else if (previousPhase.current === 'choice') {
      invoke('hide_pie', { approval: state.phase === 'approval' }).catch(() => {});
    }
    previousPhase.current = state.phase;
  }, [state.phase, pieWindow]);

  const menuVisible = state.phase === 'choice' && plan && (!nativePie || pieWindow);
  const approvalVisible = state.phase === 'approval' && state.approval && !pieWindow;
  const modalVisible = Boolean(menuVisible || approvalVisible);

  const interaction = (
    <>
      {menuVisible && (
        <Dialog
          title="What would you like to do?"
          eyebrow="YOUR CONTEXT. YOUR CHOICE."
          onCancel={() => choose(null)}
          className="pie-dialog"
          bare={pieWindow}
        >
          {!pieWindow && (
            <p className="dialog-description">
              Suggestions from Bloom’s{' '}
              {plan.decided_by === 'rules-v0' ? 'application-based rules' : plan.decided_by}.
            </p>
          )}
          <PieMenu
            plan={plan}
            onSelect={choose}
            onCancel={() => choose(null)}
            onShown={onShown}
            rotation={prefs.rotation}
            disabled={submitted || !connected}
            dismissOnBackground={pieWindow}
          />
          {!pieWindow && (
            <div className="pie-help">
              <span>
                <kbd>1–{plan.candidates.length}</kbd> select
              </span>
              <span>
                <kbd>↑</kbd>
                <kbd>↓</kbd> navigate
              </span>
              <span>
                <kbd>↵</kbd> confirm
              </span>
              <span>
                <kbd>esc</kbd> cancel
              </span>
            </div>
          )}
          {!pieWindow && (
            <p className="dialog-footnote">
              Simulated context · Your selection is sent to the Python core
            </p>
          )}
        </Dialog>
      )}
      {approvalVisible && (
        <Dialog
          title="Ready when you are."
          eyebrow="APPROVAL REQUIRED"
          onCancel={() => respond('reject')}
          className="approval-dialog"
        >
          <ApprovalPanel
            request={state.approval!}
            onRespond={respond}
            disabled={submitted || !connected}
          />
        </Dialog>
      )}
    </>
  );

  if (pieWindow)
    return (
      <div className="floating-pie-root">
        {interaction}
        {!connected && (
          <div className="floating-disconnected" role="alert">
            Bloom is disconnected. Return to the main window.
          </div>
        )}
      </div>
    );

  function update<K extends keyof Preferences>(key: K, value: Preferences[K]) {
    setPrefs((old) => ({ ...old, [key]: value }));
  }

  return (
    <>
      <div className="app-layout" inert={modalVisible || undefined}>
        <aside className="sidebar">
          <a
            className="brand"
            href="#"
            onClick={(event) => {
              event.preventDefault();
              setPage('activity');
            }}
            aria-label="Bloom home"
          >
            <BloomMark />
            <span>
              bloom<span className="brand-period">.</span>
            </span>
          </a>
          <div className="workspace-label">YOUR WORKSPACE</div>
          <nav aria-label="Main navigation">
            {(
              [
                { id: 'activity', icon: 'activity', label: 'Activity' },
                { id: 'pie', icon: 'pie', label: 'Pie menu' },
                { id: 'settings', icon: 'settings', label: 'Preferences' },
              ] as const
            ).map((item) => (
              <button
                key={item.id}
                className={`nav-item ${page === item.id ? 'active' : ''}`}
                aria-current={page === item.id ? 'page' : undefined}
                onClick={() => setPage(item.id)}
              >
                <Icon name={item.icon} size={19} />
                <span>{item.label}</span>
                {page === item.id && <span className="nav-dot" />}
              </button>
            ))}
          </nav>
          <div className="sidebar-bottom">
            <div className="local-card">
              <span className={`status-dot ${connected ? 'online' : ''}`} />
              <div>
                <strong>
                  {connected
                    ? 'Core connected'
                    : connection === 'connecting'
                      ? 'Connecting…'
                      : 'Core disconnected'}
                </strong>
                <small>Running on your device</small>
              </div>
            </div>
            <div className="version">
              <span>Bloom Core</span>
              <span>
                v0.0.1 <span className="alpha">DEV</span>
              </span>
            </div>
          </div>
        </aside>
        <div className="main-shell">
          <header className="topbar">
            <div className="breadcrumb">
              Workspace <span>/</span>{' '}
              <strong>
                {page === 'activity' ? 'Overview' : page === 'pie' ? 'Pie menu' : 'Preferences'}
              </strong>
            </div>
            <div className="session-badge">
              <span /> LOCAL DEMO SESSION
            </div>
          </header>
          <main className="main-content">
            {(error || nativeError) && (
              <div className="error-banner" role="alert">
                <Icon name="info" size={18} />
                <span>{error ?? nativeError}</span>
                <button
                  className="icon-button"
                  aria-label="Dismiss notification"
                  onClick={() => {
                    dismissError();
                    setNativeError(null);
                  }}
                >
                  <Icon name="close" size={16} />
                </button>
              </div>
            )}
            {!connected && (
              <div className="connection-notice" role="status">
                <Icon name="terminal" size={18} />
                <span>
                  {connection === 'missing' ? (
                    <>
                      Start <code>make ui</code> and open the launch link printed in your terminal
                      to connect.
                    </>
                  ) : (
                    <>
                      Waiting for the Python core. Your activity will appear when the connection is
                      restored.
                    </>
                  )}
                </span>
              </div>
            )}
            {page === 'activity' && (
              <>
                <section className="hero">
                  <div className="hero-copy">
                    <p className="eyebrow">
                      <span className="tiny-flower">✳</span> A LITTLE LESS FRICTION
                    </p>
                    <h1>
                      A little closer to
                      <br />
                      <span>your next action.</span>
                    </h1>
                    <p className="hero-description">
                      The right suggestion, in the right moment.
                      <br />
                      You choose what happens next.
                    </p>
                    <div className="hero-actions">
                      <button
                        className="button primary"
                        onClick={() => start()}
                        disabled={!connected || busy}
                      >
                        <Icon name="play" size={16} />
                        {busy ? 'Demo in progress' : 'Start a demo'}
                        <span className="button-shortcut">{shortcutLabel}</span>
                      </button>
                      <span className="hero-note">A safe place to try Bloom.</span>
                    </div>
                  </div>
                  <div className="hero-art" aria-hidden="true">
                    <div className="art-orbit orbit-one" />
                    <div className="art-orbit orbit-two" />
                    <div className="art-orbit orbit-three" />
                    <BloomMark />
                    <span className="art-dot dot-one" />
                    <span className="art-dot dot-two" />
                    <span className="art-label">CONTEXT → POSSIBILITY</span>
                  </div>
                </section>
                <section className="session-overview" aria-label="Session overview">
                  <div className="overview-item">
                    <span className="overview-icon">
                      <Icon name="terminal" />
                    </span>
                    <div>
                      <span>DECISION ENGINE</span>
                      <strong>{latestPlan?.decided_by ?? 'Application-based rules'}</strong>
                    </div>
                    <span className="subtle-tag">Local</span>
                  </div>
                  <div className="overview-item">
                    <span className="overview-icon">
                      <Icon name="pie" />
                    </span>
                    <div>
                      <span>CONTEXTUAL SUGGESTIONS</span>
                      <strong>
                        {latestPlan
                          ? `${latestPlan.candidates.length} actions in your pie`
                          : 'Ready for your first run'}
                      </strong>
                    </div>
                  </div>
                  <div className="overview-item">
                    <span className="overview-icon">
                      <Icon name="shield" />
                    </span>
                    <div>
                      <span>HUMAN CONTROL</span>
                      <strong>You approve risky actions</strong>
                    </div>
                  </div>
                </section>
                {busy && (
                  <div className="live-run" role="status">
                    <span className="status-dot online" />
                    <strong>
                      {state.phase === 'starting'
                        ? 'Preparing your suggestions…'
                        : state.phase === 'choice'
                          ? 'Waiting for your choice'
                          : 'Waiting for your approval'}
                    </strong>
                    <span>The Python run is paused for your input.</span>
                  </div>
                )}
                <div className="overview-grid">
                  <ActivityView history={state.history} warning={state.history_warning} />
                  <aside className="explanation-card">
                    <div className="explanation-icon">
                      <BloomMark />
                    </div>
                    <p className="eyebrow">UNDER THE SURFACE</p>
                    <h2>
                      Small steps.
                      <br />
                      Clear intentions.
                    </h2>
                    <p>Follow a demo from context to action.</p>
                    <ol className="flow-steps">
                      <li>
                        <span>01</span>
                        <div>
                          <strong>Gather context</strong>
                          <p>A fictional email in Apple Mail.</p>
                        </div>
                      </li>
                      <li>
                        <span>02</span>
                        <div>
                          <strong>Suggest a next step</strong>
                          <p>Rules rank the available actions.</p>
                        </div>
                      </li>
                      <li>
                        <span>03</span>
                        <div>
                          <strong>You make the call</strong>
                          <p>Choose, review, approve or cancel.</p>
                        </div>
                      </li>
                      <li>
                        <span>04</span>
                        <div>
                          <strong>Keep a clear record</strong>
                          <p>The outcome is saved locally.</p>
                        </div>
                      </li>
                    </ol>
                    <div className="demo-note">
                      <Icon name="info" size={16} />
                      <p>
                        This is a connected prototype. Screen capture and action execution are
                        simulated.
                      </p>
                    </div>
                  </aside>
                </div>
              </>
            )}
            {page === 'pie' && (
              <>
                <div className="page-heading">
                  <p className="eyebrow">MAKE ROOM FOR WHAT’S NEXT</p>
                  <h1>Your pie. Your flow.</h1>
                  <p>Actions come from the core. Make the way you reach them feel natural.</p>
                </div>
                <div className="studio-grid">
                  <section className="studio-preview">
                    <div className="card-heading">
                      <div>
                        <h2>Menu preview</h2>
                        <p>
                          {latestPlan
                            ? `Actual suggestions from ${latestPlan.decided_by}`
                            : 'Start a demo to receive your first suggestions.'}
                        </p>
                      </div>
                      <span className="subtle-tag">Preview</span>
                    </div>
                    {latestPlan ? (
                      <PieMenu
                        plan={latestPlan}
                        rotation={prefs.rotation}
                        onSelect={(id) =>
                          setPreviewSelection(
                            latestPlan.candidates.find((candidate) => candidate.action_id === id)
                              ?.label ?? id,
                          )
                        }
                      />
                    ) : (
                      <div className="preview-empty">
                        <BloomMark />
                        <p>Your suggestions will bloom here.</p>
                      </div>
                    )}
                    <p className="preview-caption" role="status">
                      {previewSelection
                        ? `Preview selection: ${previewSelection}. Start a demo to submit a real choice.`
                        : 'Preview interactions do not submit an action.'}
                    </p>
                  </section>
                  <section className="settings-card">
                    <p className="eyebrow">LAYOUT</p>
                    <h2>A familiar starting point.</h2>
                    <p>Keep the first suggestion in a position you can remember.</p>
                    <div className="segmented-control" aria-label="First action position">
                      <button
                        aria-pressed={prefs.rotation === -90}
                        onClick={() => update('rotation', -90)}
                      >
                        At the top
                      </button>
                      <button
                        aria-pressed={prefs.rotation === 0}
                        onClick={() => update('rotation', 0)}
                      >
                        On the right
                      </button>
                    </div>
                    <div className="settings-divider" />
                    <h3>Built around your context</h3>
                    <p>
                      The number, labels and order of suggestions are supplied by the Python core.
                      Changing the model will feed this same menu.
                    </p>
                    <button
                      className="button primary full-width"
                      onClick={() => start()}
                      disabled={!connected || busy}
                    >
                      <Icon name="play" size={16} />
                      Try a live demo
                    </button>
                    <p className="settings-footnote">
                      Use arrows to navigate, Enter to choose, or Escape to close.
                    </p>
                  </section>
                </div>
              </>
            )}
            {page === 'settings' && (
              <>
                <div className="page-heading">
                  <p className="eyebrow">A SPACE THAT FEELS LIKE YOU</p>
                  <h1>A few thoughtful defaults.</h1>
                  <p>Preferences are saved on this device.</p>
                </div>
                <section className="preferences-card">
                  <div className="preference-row">
                    <div>
                      <h3>Accent color</h3>
                      <p>A quiet touch of color throughout Bloom.</p>
                    </div>
                    <div className="accent-options">
                      <button
                        className="color-option sage"
                        aria-label="Sage accent"
                        aria-pressed={prefs.accent === 'sage'}
                        onClick={() => update('accent', 'sage')}
                      >
                        {prefs.accent === 'sage' && <Icon name="check" size={17} />}
                      </button>
                      <button
                        className="color-option lilac"
                        aria-label="Lilac accent"
                        aria-pressed={prefs.accent === 'lilac'}
                        onClick={() => update('accent', 'lilac')}
                      >
                        {prefs.accent === 'lilac' && <Icon name="check" size={17} />}
                      </button>
                    </div>
                  </div>
                  <div className="preference-row">
                    <div>
                      <h3>Reduce motion</h3>
                      <p>Keep transitions and decorative movement still.</p>
                    </div>
                    <input
                      className="toggle"
                      type="checkbox"
                      aria-label="Reduce motion"
                      checked={prefs.reducedMotion}
                      onChange={(event) => update('reducedMotion', event.target.checked)}
                    />
                  </div>
                  <div className="preference-row">
                    <div>
                      <h3>Quick launch</h3>
                      <p>
                        <kbd>{shortcutLabel}</kbd> opens the pie menu.
                        <br />
                        {shortcutStatus.message}
                      </p>
                    </div>
                    <input
                      className="toggle"
                      type="checkbox"
                      aria-label="Enable quick launch"
                      checked={prefs.shortcut}
                      onChange={(event) => update('shortcut', event.target.checked)}
                    />
                  </div>
                  <div className="preference-row">
                    <div>
                      <h3>Connection</h3>
                      <p>The interface communicates with a Python core on your device.</p>
                    </div>
                    <span className={`connection-label ${connected ? 'connected' : ''}`}>
                      <span className={`status-dot ${connected ? 'online' : ''}`} />
                      {connected ? 'Connected' : 'Disconnected'}
                    </span>
                  </div>
                </section>
                <div className="settings-info">
                  <Icon name="info" size={18} />
                  <p>
                    The pie floats above your workspace. Escape, the centre button or clicking away
                    dismisses it. Approval brings you back to this window.
                  </p>
                </div>
              </>
            )}
            <footer className="main-footer">
              <span>
                <BloomMark /> Thoughtful assistance. Human control.
              </span>
              <span>Built to stay close.</span>
            </footer>
          </main>
        </div>
      </div>
      {interaction}
    </>
  );
}
