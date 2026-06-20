// DR-02/DR-03: the model-grid + model-card grammar from SceneWorks'
// ModelManagerScreen. One card per model: title + single status badge, optional
// description, a definition list of evidence/meta, and an action row. The Models
// page (DR-03) groups these into collapsible model-type-group sections.
import { Fragment } from "react";
import type { ReactNode } from "react";

function cx(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

/** Responsive auto-fill grid of model cards. */
export function ModelGrid({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={cx("model-grid", className)}>{children}</div>;
}

export interface ModelCardMeta {
  label: ReactNode;
  value: ReactNode;
}

/**
 * A single model card. `status` is a rendered chip (e.g. <StatusBadge/>),
 * `meta` becomes a <dl>, `actions` the action row, and `children` renders below
 * (e.g. a gated-availability notice).
 */
export function ModelCard({
  title,
  status,
  description,
  meta,
  actions,
  children,
  className,
}: {
  title: ReactNode;
  status?: ReactNode;
  description?: ReactNode;
  meta?: ReadonlyArray<ModelCardMeta>;
  actions?: ReactNode;
  children?: ReactNode;
  className?: string;
}) {
  return (
    <article className={cx("main-surface", "model-card", className)}>
      <div className="model-card-heading">
        <h3>{title}</h3>
        {status}
      </div>
      {description ? <p>{description}</p> : null}
      {meta && meta.length > 0 ? (
        <dl>
          {meta.map((entry, index) => (
            <Fragment key={index}>
              <dt>{entry.label}</dt>
              <dd>{entry.value}</dd>
            </Fragment>
          ))}
        </dl>
      ) : null}
      {actions ? <div className="model-card-actions">{actions}</div> : null}
      {children}
    </article>
  );
}
