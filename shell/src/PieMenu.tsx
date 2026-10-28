import { useEffect, useRef, useState } from 'react';
import type { ActionPlan } from './contracts.generated';
import { actionIcon, BloomMark, Icon } from './Icons';

const CENTER = 240;
function point(radius: number, degrees: number) {
  const angle = (degrees * Math.PI) / 180;
  return [CENTER + radius * Math.cos(angle), CENTER + radius * Math.sin(angle)];
}

export function sectorPath(start: number, end: number) {
  const outerStart = point(212, start);
  const outerEnd = point(212, end);
  const innerEnd = point(91, end);
  const innerStart = point(91, start);
  const large = end - start > 180 ? 1 : 0;
  return `M ${outerStart} A 212 212 0 ${large} 1 ${outerEnd} L ${innerEnd} A 91 91 0 ${large} 0 ${innerStart} Z`;
}

type Props = {
  plan: ActionPlan;
  onSelect: (id: string) => void;
  onCancel?: () => void;
  onShown?: (at: string) => void;
  rotation?: number;
  disabled?: boolean;
  dismissOnBackground?: boolean;
};

export function PieMenu({
  plan,
  onSelect,
  onCancel,
  onShown,
  rotation = -90,
  disabled = false,
  dismissOnBackground = false,
}: Props) {
  const [active, setActive] = useState(0);
  const sectors = useRef<Array<SVGGElement | null>>([]);
  const count = plan.candidates.length;

  useEffect(() => {
    setActive(0);
    let second = 0;
    const first = requestAnimationFrame(() => {
      second = requestAnimationFrame(() => onShown?.(new Date().toISOString()));
    });
    return () => {
      cancelAnimationFrame(first);
      cancelAnimationFrame(second);
    };
  }, [plan.request_id, onShown]);

  function navigate(event: React.KeyboardEvent) {
    if (disabled || count === 0) return;
    let next = active;
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') next = (active + 1) % count;
    else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp')
      next = (active - 1 + count) % count;
    else if (/^[1-9]$/.test(event.key) && Number(event.key) <= count) {
      event.preventDefault();
      onSelect(plan.candidates[Number(event.key) - 1].action_id);
      return;
    } else return;
    event.preventDefault();
    setActive(next);
    sectors.current[next]?.focus();
  }

  return (
    <div
      className={`pie-stage ${disabled ? 'is-pending' : ''}`}
      onKeyDown={navigate}
      onPointerDown={(event) => {
        const target = event.target as Element;
        if (dismissOnBackground && !disabled && !target.closest('.pie-sector, .pie-center'))
          onCancel?.();
      }}
    >
      <svg className="pie-svg" viewBox="0 0 480 480" aria-label="Suggested actions">
        <circle cx="240" cy="240" r="226" className="pie-orbit" />
        <circle cx="240" cy="240" r="221" className="pie-orbit inner" />
        {plan.candidates.map((candidate, index) => {
          const angle = rotation + (index * 360) / count;
          const [x, y] = point(148, angle);
          const words = candidate.label.split(' ');
          const lines =
            candidate.label.length > 18
              ? [
                  words.slice(0, Math.ceil(words.length / 2)).join(' '),
                  words.slice(Math.ceil(words.length / 2)).join(' '),
                ]
              : [candidate.label];
          return (
            <g
              key={candidate.action_id}
              ref={(node) => {
                sectors.current[index] = node;
              }}
              role="button"
              tabIndex={disabled ? -1 : 0}
              aria-label={candidate.label}
              aria-disabled={disabled}
              className={`pie-sector ${active === index ? 'is-active' : ''}`}
              onPointerEnter={() => setActive(index)}
              onFocus={() => setActive(index)}
              onClick={() => {
                if (!disabled) onSelect(candidate.action_id);
              }}
              onKeyDown={(event) => {
                if ((event.key === 'Enter' || event.key === ' ') && !disabled) {
                  event.preventDefault();
                  onSelect(candidate.action_id);
                }
              }}
            >
              <path d={sectorPath(angle - 180 / count + 1.2, angle + 180 / count - 1.2)} />
              <Icon name={actionIcon(candidate.action_id)} x={x - 11} y={y - 35} size={22} />
              <text x={x} y={y + 5} textAnchor="middle" className="sector-label">
                {lines.map((line, i) => (
                  <tspan key={i} x={x} dy={i ? 19 : 0}>
                    {line}
                  </tspan>
                ))}
              </text>
              <text
                x={x}
                y={y + (lines.length > 1 ? 54 : 38)}
                textAnchor="middle"
                className="sector-key"
              >
                0{index + 1}
              </text>
            </g>
          );
        })}
      </svg>
      {onCancel ? (
        <button
          className="pie-center"
          onClick={onCancel}
          aria-label="Cancel action selection"
          disabled={disabled}
        >
          <BloomMark />
          <span>bloom</span>
          <small>esc to close</small>
        </button>
      ) : (
        <div className="pie-center">
          <BloomMark />
          <span>bloom</span>
          <small>your next action</small>
        </div>
      )}
      {count === 0 && (
        <p className="pie-empty">No actions available. Close this menu and try again.</p>
      )}
    </div>
  );
}
