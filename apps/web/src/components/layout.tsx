// DR-02: shared layout primitives mirroring SceneWorks' component grammar
// (.main-surface / .surface-header.hero / .section-heading / .toolbar /
// .segmented-control / .status-badge / .empty-panel). Screens compose these
// instead of re-inventing one-off classes, so spacing/headers/badges/cards
// stay consistent across the app.
import type { ReactNode } from "react";

function cx(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

/** A screen/section container: padded card surface every screen body sits in. */
export function MainSurface({
  children,
  className,
  ariaLabel,
}: {
  children: ReactNode;
  className?: string;
  ariaLabel?: string;
}) {
  return (
    <section className={cx("main-surface", className)} aria-label={ariaLabel}>
      {children}
    </section>
  );
}

/** Eyebrow + heading pair used by surface headers and sub-panels. */
export function SectionHeading({
  eyebrow,
  title,
  className,
}: {
  eyebrow?: ReactNode;
  title: ReactNode;
  className?: string;
}) {
  return (
    <div className={cx("section-heading", className)}>
      {eyebrow ? <p className="eyebrow">{eyebrow}</p> : null}
      <h2>{title}</h2>
    </div>
  );
}

/**
 * Screen header. Defaults to the gradient `hero` variant (eyebrow + title +
 * blurb on the left, actions on the right, optional stat row spanning below).
 * Pass `hero={false}` for a plain flush header.
 */
export function SurfaceHeader({
  eyebrow,
  title,
  blurb,
  actions,
  stats,
  hero = true,
  className,
}: {
  eyebrow?: ReactNode;
  title: ReactNode;
  blurb?: ReactNode;
  actions?: ReactNode;
  stats?: ReactNode;
  hero?: boolean;
  className?: string;
}) {
  return (
    <header className={cx("surface-header", hero && "hero", className)}>
      <div className="section-heading">
        {eyebrow ? <p className="eyebrow">{eyebrow}</p> : null}
        <h2>{title}</h2>
        {blurb ? <p className="hero-blurb">{blurb}</p> : null}
      </div>
      {actions ? <div className="surface-header-actions">{actions}</div> : null}
      {stats ? <div className="hero-stats">{stats}</div> : null}
    </header>
  );
}

/** A single stat tile for the SurfaceHeader `stats` row. */
export function HeroStat({
  label,
  value,
}: {
  label: ReactNode;
  value: ReactNode;
}) {
  return (
    <div className="hero-stat">
      <span className="hero-stat-label">{label}</span>
      <span className="hero-stat-value">{value}</span>
    </div>
  );
}

/** Flex-wrap row for action buttons / filters. */
export function Toolbar({
  children,
  className,
  ariaLabel,
}: {
  children: ReactNode;
  className?: string;
  ariaLabel?: string;
}) {
  return (
    <div className={cx("toolbar", className)} role="group" aria-label={ariaLabel}>
      {children}
    </div>
  );
}

export interface SegmentedOption<T extends string> {
  value: T;
  label: ReactNode;
  disabled?: boolean;
}

/** Mode/tab switch. Generic over the option id type. */
export function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
  compact = false,
  ariaLabel,
}: {
  options: ReadonlyArray<SegmentedOption<T>>;
  value: T;
  onChange: (next: T) => void;
  compact?: boolean;
  ariaLabel?: string;
}) {
  return (
    <div
      className={compact ? "compact-segment" : "segmented-control"}
      role="group"
      aria-label={ariaLabel}
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          className={option.value === value ? "active" : undefined}
          aria-pressed={option.value === value}
          disabled={option.disabled}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export type StatusTone =
  | "neutral"
  | "installed"
  | "completed"
  | "warning"
  | "failed"
  | "canceled"
  | "interrupted";

/** Single status chip with a tone modifier (parity with SceneWorks states). */
export function StatusBadge({
  tone = "neutral",
  children,
  className,
}: {
  tone?: StatusTone;
  children: ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cx("status-badge", tone !== "neutral" && tone, className)}
    >
      {children}
    </span>
  );
}

/** Dashed empty/placeholder panel. `compact` tightens the padding. */
export function EmptyPanel({
  children,
  compact = false,
  className,
}: {
  children: ReactNode;
  compact?: boolean;
  className?: string;
}) {
  return (
    <div className={cx("empty-panel", compact && "compact-panel", className)}>
      {children}
    </div>
  );
}
