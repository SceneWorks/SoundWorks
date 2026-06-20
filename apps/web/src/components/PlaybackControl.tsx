// UX-F2: shared inline audio playback. Wraps the native <audio> element so any
// screen with a LibraryPlayback can audition a clip consistently — previously
// only LibraryScreen rendered the element, so ReviewScreen's Play button set the
// playback state but had nothing to play it. Renders nothing until a playback is
// loaded; shows the reason when a clip is not playable.
import type { LibraryPlayback } from "../types";

export function PlaybackControl({
  playback,
  className,
}: {
  playback: LibraryPlayback | null;
  className?: string;
}) {
  if (!playback) {
    return null;
  }
  if (!playback.playable || !playback.path) {
    return (
      <p
        className={["playback-control", "unavailable", className]
          .filter(Boolean)
          .join(" ")}
      >
        {playback.reason ?? "This asset has no playable audio yet."}
      </p>
    );
  }
  return (
    <audio
      className={["playback-control", className].filter(Boolean).join(" ")}
      controls
      preload="none"
      src={playback.path}
    >
      <track kind="captions" />
    </audio>
  );
}
