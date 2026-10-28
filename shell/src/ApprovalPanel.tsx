import { useEffect, useState } from 'react';
import type { ApprovalRequest } from './contracts.generated';
import { Icon } from './Icons';

export const riskLabel = (level: number | null | undefined) =>
  level == null
    ? 'Not assessed'
    : (['Read only', 'Reversible', 'Destructive', 'External'][level] ?? 'Unknown');

export function ApprovalPanel({
  request,
  onRespond,
  disabled = false,
}: {
  request: ApprovalRequest;
  onRespond: (verdict: 'approve' | 'reject') => void;
  disabled?: boolean;
}) {
  const [remaining, setRemaining] = useState(() =>
    Math.max(0, Date.parse(request.expires_at) - Date.now()),
  );
  useEffect(() => {
    function tick() {
      const left = Math.max(0, Date.parse(request.expires_at) - Date.now());
      setRemaining(left);
      if (left === 0) onRespond('reject');
    }
    tick();
    const timer = setInterval(tick, 1000);
    return () => clearInterval(timer);
  }, [request.run_id, request.expires_at, onRespond]);
  return (
    <div className="approval-content">
      <div className="approval-notice">
        <Icon name="shield" size={24} />
        <div>
          <strong>You have the final say.</strong>
          <p>Review the prepared action before continuing.</p>
        </div>
        <span className="risk-tag">{riskLabel(request.effective)}</span>
      </div>
      <div className="preview-box">
        <span className="eyebrow">PREPARED ACTION</span>
        <p>{request.preview}</p>
        <span className="simulation-label">Demo action · execution is simulated</span>
      </div>
      <div className="approval-reasons">
        <h3>Why Bloom is asking</h3>
        {request.reasons.map((reason) => (
          <p key={reason}>
            <Icon name="info" size={16} />
            {reason}
          </p>
        ))}
      </div>
      <p className="expiry">
        <Icon name="clock" size={15} /> Automatically rejected in {Math.ceil(remaining / 1000)}{' '}
        seconds.
      </p>
      <div className="approval-actions">
        <button
          className="button secondary"
          data-initial-focus
          onClick={() => onRespond('reject')}
          disabled={disabled}
        >
          Reject action
        </button>
        <button
          className="button primary"
          onClick={() => onRespond('approve')}
          disabled={disabled || remaining === 0}
        >
          <Icon name="check" size={17} />
          Approve action
        </button>
      </div>
    </div>
  );
}
