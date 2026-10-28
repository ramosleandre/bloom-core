import { useLayoutEffect, useRef } from 'react';
import type { ReactNode } from 'react';
import { Icon } from './Icons';

export function Dialog({
  title,
  eyebrow,
  children,
  onCancel,
  className = '',
  bare = false,
}: {
  title: string;
  eyebrow: string;
  children: ReactNode;
  onCancel: () => void;
  className?: string;
  bare?: boolean;
}) {
  const panel = useRef<HTMLDivElement>(null);
  const previous = useRef(document.activeElement);
  const cancel = useRef(onCancel);
  cancel.current = onCancel;
  useLayoutEffect(() => {
    const target =
      panel.current?.querySelector<HTMLElement>('[data-initial-focus]') ??
      panel.current?.querySelector<HTMLElement>('[role="button"][tabindex="0"]') ??
      panel.current?.querySelector<HTMLElement>('button');
    target?.focus();
    const escape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        cancel.current();
      }
    };
    document.addEventListener('keydown', escape, true);
    return () => {
      document.removeEventListener('keydown', escape, true);
      if (previous.current instanceof HTMLElement && previous.current.isConnected)
        previous.current.focus();
    };
  }, []);
  return (
    <div
      className="modal-backdrop"
      onPointerDown={(event) => {
        if (event.target === event.currentTarget) onCancel();
      }}
    >
      <div
        className={`dialog ${bare ? 'radial-dialog' : ''} ${className}`}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        ref={panel}
        onKeyDown={(event) => {
          if (event.key === 'Tab') {
            const nodes = panel.current?.querySelectorAll<HTMLElement>(
              'button:not([disabled]), [tabindex="0"]',
            );
            if (!nodes?.length) return;
            const first = nodes[0];
            const last = nodes[nodes.length - 1];
            if (event.shiftKey && document.activeElement === first) {
              event.preventDefault();
              last.focus();
            } else if (!event.shiftKey && document.activeElement === last) {
              event.preventDefault();
              first.focus();
            }
          }
        }}
      >
        {!bare && (
          <header className="dialog-heading">
            <div>
              <p className="eyebrow">{eyebrow}</p>
              <h2>{title}</h2>
            </div>
            <button
              className="icon-button close-dialog"
              aria-label="Close dialog"
              onClick={onCancel}
            >
              <Icon name="close" />
            </button>
          </header>
        )}
        {children}
      </div>
    </div>
  );
}
