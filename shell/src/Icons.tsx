import type { SVGProps } from 'react';

export function BloomMark({ className = '', ...props }: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 32 32" fill="none" className={className} aria-hidden="true" {...props}>
      {[0, 60, 120, 180, 240, 300].map((angle) => (
        <ellipse
          key={angle}
          cx="16"
          cy="9.5"
          rx="4.1"
          ry="7.2"
          transform={`rotate(${angle} 16 16)`}
          stroke="currentColor"
          strokeWidth="1.4"
        />
      ))}
      <circle cx="16" cy="16" r="2.6" fill="currentColor" />
    </svg>
  );
}

const paths: Record<string, React.ReactNode> = {
  activity: (
    <>
      <path d="M3 12h4l3-8 4 16 3-8h4" />
    </>
  ),
  pie: (
    <>
      <path d="M12 3v9h9A9 9 0 1 1 12 3Z" />
      <path d="M16 3.8A9 9 0 0 1 20.2 8H16Z" />
    </>
  ),
  settings: (
    <>
      <path d="m9 3-.6 2.3-2 .9L4.2 6l-2 3.4L4 11v2l-1.8 1.6 2 3.4 2.2-.2 2 .9L9 21h4l.6-2.3 2-.9 2.2.2 2-3.4L18 13v-2l1.8-1.6-2-3.4-2.2.2-2-.9L13 3Z" />
      <circle cx="11" cy="12" r="3" />
    </>
  ),
  arrow: (
    <>
      <path d="M5 12h14m-6-6 6 6-6 6" />
    </>
  ),
  play: <path d="m9 5 11 7-11 7Z" />,
  check: <path d="m5 12 4 4L19 6" />,
  close: <path d="m6 6 12 12M6 18 18 6" />,
  shield: (
    <>
      <path d="m12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6Z" />
      <path d="m8 12 3 3 5-6" />
    </>
  ),
  clock: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7v5l3 2" />
    </>
  ),
  mail: (
    <>
      <rect x="3" y="5" width="18" height="14" rx="3" />
      <path d="m4 7 8 6 8-6" />
    </>
  ),
  summary: (
    <>
      <rect x="5" y="3" width="14" height="18" rx="2" />
      <path d="M9 8h6M9 12h6M9 16h3" />
    </>
  ),
  calendar: (
    <>
      <rect x="3" y="5" width="18" height="16" rx="3" />
      <path d="M7 3v4m10-4v4M3 11h18M8 15h2m4 0h2" />
    </>
  ),
  translate: (
    <>
      <path d="M3 5h12M9 3v2m4 0c0 7-8 11-10 12M5 8c1 3 4 6 8 8m1 5 4-10 4 10m-6-4h4" />
    </>
  ),
  code: (
    <>
      <path d="m8 6-6 6 6 6m8-12 6 6-6 6m-3-14-2 16" />
    </>
  ),
  note: (
    <>
      <path d="M13 3H5v18h14V9Z" />
      <path d="M13 3v6h6M9 13h6m-6 4h4" />
    </>
  ),
  trash: (
    <>
      <path d="M3 6h18M9 3h6m-10 3 1 15h12l1-15M10 10v7m4-7v7" />
    </>
  ),
  search: (
    <>
      <circle cx="10" cy="10" r="6" />
      <path d="m15 15 5 5" />
    </>
  ),
  info: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 11v6m0-10v.1" />
    </>
  ),
  terminal: (
    <>
      <rect x="3" y="4" width="18" height="16" rx="3" />
      <path d="m7 9 3 3-3 3m6 0h4" />
    </>
  ),
};

export function Icon({
  name,
  size = 20,
  ...props
}: SVGProps<SVGSVGElement> & { name: string; size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {paths[name] ?? paths.arrow}
    </svg>
  );
}

export function actionIcon(id: string) {
  if (id.startsWith('reply.')) return 'mail';
  if (id.includes('summarize')) return 'summary';
  if (id.includes('translate')) return 'translate';
  if (id.startsWith('calendar.')) return 'calendar';
  if (id.startsWith('code.')) return 'code';
  if (id.startsWith('file.')) return 'trash';
  return 'note';
}
