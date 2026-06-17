import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("renders the scaffolded SoundWorks workspace", async () => {
    render(<App />);

    expect(screen.getByRole("main")).toBeInTheDocument();
    expect(screen.getAllByText("SoundWorks").length).toBeGreaterThan(0);
    expect(
      await screen.findByRole("button", { name: /TTS Studio/i }),
    ).toBeInTheDocument();
    expect(screen.getByText("get_app_overview")).toBeInTheDocument();
    expect(screen.getByText("Provider Coverage")).toBeInTheDocument();
    expect(screen.getByText("Stem Separation")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Worker Runtime" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Python runtime: blocked")).toBeInTheDocument();
    expect(screen.getByText("Reference Speech Suite")).toBeInTheDocument();
    expect(
      screen.getByText(/Reinstall the provider package/i),
    ).toBeInTheDocument();
  });
});
