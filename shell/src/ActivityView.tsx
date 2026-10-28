import { useState } from 'react';
import type { RunState } from './contracts.generated';
import { riskLabel } from './ApprovalPanel';
import { actionIcon, Icon } from './Icons';

export function runLabel(run: RunState) {
  if (run.chosen)
    return (
      run.plan?.candidates.find((c) => c.action_id === run.chosen?.action_id)?.label ??
      run.chosen.action_id
    );
  return run.status === 'cancelled' ? 'Menu dismissed' : 'Demo run';
}

function outcome(run: RunState) {
  if (run.status === 'done') return run.result?.ok === false ? 'failed' : 'completed';
  return run.status === 'cancelled' ? 'cancelled' : 'failed';
}

export function ActivityView({
  history,
  warning,
}: {
  history: RunState[];
  warning?: string | null;
}) {
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  const runs = history.filter(
    (run) =>
      (filter === 'all' || outcome(run) === filter) &&
      `${runLabel(run)} ${run.run_id} ${run.plan?.decided_by ?? ''}`
        .toLowerCase()
        .includes(search.toLowerCase()),
  );
  return (
    <section className="activity-card" aria-labelledby="activity-heading">
      <div className="card-heading">
        <div>
          <h2 id="activity-heading">
            Recent activity <span className="count">{history.length}</span>
          </h2>
          <p>A clear view of what happened, and why.</p>
        </div>
        <Icon name="clock" size={18} />
      </div>
      <div className="activity-tools">
        <div className="filter-tabs" aria-label="Filter activity">
          {['all', 'completed', 'cancelled', 'failed'].map((value) => (
            <button
              key={value}
              aria-pressed={filter === value}
              className={filter === value ? 'selected' : ''}
              onClick={() => setFilter(value)}
            >
              {value[0].toUpperCase() + value.slice(1)}
            </button>
          ))}
        </div>
        <label className="search-field">
          <Icon name="search" size={16} />
          <input
            aria-label="Search activity"
            value={search}
            placeholder="Search activity"
            onChange={(event) => setSearch(event.target.value)}
          />
        </label>
      </div>
      {warning && (
        <p className="history-warning" role="status">
          {warning}
        </p>
      )}
      <div className="activity-columns">
        <span>ACTION</span>
        <span>OUTCOME</span>
        <span>TIME</span>
      </div>
      {runs.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon">
            <Icon name="activity" size={28} />
          </div>
          <h3>{history.length ? 'No matching activity' : 'Your next action starts here.'}</h3>
          <p>
            {history.length
              ? 'Try a different filter or search.'
              : 'Start a demo and choose an action. Its story will appear here.'}
          </p>
        </div>
      ) : (
        <div className="activity-list">
          {runs.map((run) => {
            const result = outcome(run);
            const expanded = selected === run.run_id;
            return (
              <div className="activity-entry" key={run.run_id}>
                <button
                  className={`activity-row ${expanded ? 'expanded' : ''}`}
                  aria-expanded={expanded}
                  onClick={() => setSelected(expanded ? null : run.run_id)}
                >
                  <span className="row-action">
                    <span className={`action-icon ${result}`}>
                      <Icon name={run.chosen ? actionIcon(run.chosen.action_id) : 'pie'} />
                    </span>
                    <span>
                      <strong>{runLabel(run)}</strong>
                      <small>
                        {run.plan?.decided_by ?? 'No decision'}
                        <span className="small-divider">·</span>Demo session
                      </small>
                    </span>
                  </span>
                  <span className={`outcome ${result}`}>
                    <span />
                    {result[0].toUpperCase() + result.slice(1)}
                  </span>
                  <time dateTime={run.trigger.at}>
                    {new Date(run.trigger.at).toLocaleTimeString([], {
                      hour: '2-digit',
                      minute: '2-digit',
                    })}
                    <Icon name="arrow" size={15} />
                  </time>
                </button>
                {expanded && (
                  <div className="run-details">
                    <div className="detail-summary">
                      <span>
                        <span className="eyebrow">RISK LEVEL</span>
                        <strong>{riskLabel(run.effective)}</strong>
                      </span>
                      <span>
                        <span className="eyebrow">HUMAN RESPONSE</span>
                        <strong>{run.approval?.verdict ?? 'Not requested'}</strong>
                      </span>
                      <span>
                        <span className="eyebrow">DATE</span>
                        <strong>{new Date(run.trigger.at).toLocaleDateString()}</strong>
                      </span>
                    </div>
                    {run.chosen && <p className="run-preview">{run.chosen.preview}</p>}
                    {run.gate_reasons.map((reason) => (
                      <p className="detail-reason" key={reason}>
                        <Icon name="shield" size={15} />
                        {reason}
                      </p>
                    ))}
                    {run.result && <p className="result-detail">{run.result.detail}</p>}
                    {run.error && <p className="run-error">{run.error}</p>}
                    <div className="timing-chips">
                      {Object.entries(run.timings_ms ?? {}).map(([step, ms]) => (
                        <span key={step}>
                          {step.replaceAll('_', ' ')} <b>{ms} ms</b>
                        </span>
                      ))}
                    </div>
                    <details className="technical-details">
                      <summary>Technical details</summary>
                      <pre>{JSON.stringify(run, null, 2)}</pre>
                    </details>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
      <footer className="activity-footer">
        <Icon name="shield" size={14} />
        <span>Activity is saved locally on your device.</span>
        <span className="muted">Latest 100 runs</span>
      </footer>
    </section>
  );
}
