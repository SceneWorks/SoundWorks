import {
  Activity,
  Boxes,
  Library,
  Mic2,
  Music2,
  Radio,
  SlidersHorizontal,
  Sparkles,
  Waves,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { fallbackOverview } from "./appData";
import { loadAppOverview } from "./tauri";
import type { AppOverview } from "./types";

const navItems = [
  { label: "Studios", icon: Sparkles },
  { label: "Library", icon: Library },
  { label: "Mixer", icon: SlidersHorizontal },
  { label: "Jobs", icon: Activity },
];

const studioIcons = [Mic2, Radio, Waves, Boxes, Music2, Sparkles];

export function App() {
  const [overview, setOverview] = useState<AppOverview>(fallbackOverview);

  useEffect(() => {
    let active = true;

    loadAppOverview().then((nextOverview) => {
      if (active) {
        setOverview(nextOverview);
      }
    });

    return () => {
      active = false;
    };
  }, []);

  const scaffoldedLayerCount = useMemo(
    () =>
      overview.architecture.layers.filter(
        (layer) => layer.status === "scaffolded",
      ).length,
    [overview.architecture.layers],
  );

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Primary">
        <div className="brand-mark" aria-label={overview.productName}>
          <Waves aria-hidden="true" size={28} />
          <span>{overview.productName}</span>
        </div>

        <nav className="nav-list">
          {navItems.map((item) => (
            <button
              className="nav-button"
              key={item.label}
              type="button"
              title={item.label}
            >
              <item.icon aria-hidden="true" size={18} />
              <span>{item.label}</span>
            </button>
          ))}
        </nav>
      </aside>

      <section className="workspace" aria-label="Workspace">
        <header className="workspace-header">
          <div>
            <p className="eyebrow">Local workspace</p>
            <h1>{overview.productName}</h1>
          </div>
          <div className="status-strip" aria-label="Scaffold status">
            <strong>{scaffoldedLayerCount}</strong>
            <span>active layers</span>
          </div>
        </header>

        <section className="studio-grid" aria-label="Studios">
          {overview.studios.map((studio, index) => {
            const Icon = studioIcons[index % studioIcons.length];

            return (
              <button className="studio-card" key={studio.id} type="button">
                <span className="icon-badge">
                  <Icon aria-hidden="true" size={22} />
                </span>
                <span className="studio-copy">
                  <strong>{studio.name}</strong>
                  <small>{studio.route}</small>
                </span>
                <span className={`state-pill ${studio.status}`}>
                  {studio.status}
                </span>
              </button>
            );
          })}
        </section>

        <section className="system-grid" aria-label="Architecture">
          <div className="panel">
            <div className="panel-heading">
              <h2>Runtime Layers</h2>
              <span>{overview.architecture.layers.length}</span>
            </div>
            <ol className="layer-list">
              {overview.architecture.layers.map((layer) => (
                <li key={layer.id}>
                  <span className={`layer-dot ${layer.status}`} />
                  <div>
                    <strong>{layer.name}</strong>
                    <p>{layer.responsibility}</p>
                  </div>
                </li>
              ))}
            </ol>
          </div>

          <div className="panel">
            <div className="panel-heading">
              <h2>Command Boundary</h2>
              <span>{overview.commands.length}</span>
            </div>
            <div className="command-list">
              {overview.commands.map((command) => (
                <article className="command-row" key={command.name}>
                  <strong>{command.name}</strong>
                  <span>{command.direction}</span>
                  <p>{command.purpose}</p>
                </article>
              ))}
            </div>
          </div>
        </section>
      </section>
    </main>
  );
}
