declare module "@sceneworks/ui" {
  import type { ComponentType, ReactNode, SVGProps } from "react";

  export interface Accent {
    id: string;
    name: string;
    swatch: string;
  }

  export const ACCENTS: Accent[];
  export const DEFAULT_ACCENT: string;
  export function isAccentId(value: unknown): value is string;

  export const version: string;

  export const Icon: Record<string, ComponentType<SVGProps<SVGSVGElement>>>;

  export function Logo(props: {
    size?: number;
    title?: string;
    className?: string;
  }): JSX.Element;

  export function StatusDot(props: { ok: boolean }): JSX.Element;

  export function Modal(props: {
    children: ReactNode;
    onClose: () => void;
    className?: string;
    labelledBy?: string;
    label?: string;
  }): JSX.Element;

  export class ErrorBoundary extends React.Component<{
    children: ReactNode;
  }> {}

  export interface CompactSelectorItem {
    id: string;
    name: string;
  }

  export function CompactSelector<T extends CompactSelectorItem>(props: {
    items?: T[];
    selectedId?: string;
    onSelect: (item: T) => void;
    onCreate?: () => void;
    createLabel?: string;
    getThumbAsset?: (item: T | null) => unknown;
    renderThumbnail?: (asset: unknown) => ReactNode;
    getSubtitle?: (item: T) => string;
    busyId?: string;
    label?: string;
    placeholder?: string;
    emptyLabel?: string;
    disabled?: boolean;
  }): JSX.Element;

  export function Markdown(props: { content: string }): JSX.Element;
}
