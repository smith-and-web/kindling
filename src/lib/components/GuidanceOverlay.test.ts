import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import GuidanceOverlay from "./GuidanceOverlay.svelte";
import { ui } from "../stores/ui.svelte";
import { currentProject } from "../stores/project.svelte";
import type { Project } from "../types";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});

beforeEach(() => {
  localStorage.clear();
  ui.setGuidanceEnabled(true);
  ui.completeOnboarding();
  currentProject.setProject({ id: "p1", name: "The Letter", project_type: "novel" } as Project);
});

afterEach(() => {
  cleanup();
  document.body.innerHTML = "";
});

function mountSidebar(right = 304) {
  const sidebar = document.createElement("aside");
  sidebar.dataset.testid = "sidebar";
  sidebar.getBoundingClientRect = () =>
    ({ top: 0, left: 0, right, bottom: 900, width: right, height: 900 }) as DOMRect;
  document.body.appendChild(sidebar);
  return sidebar;
}

/**
 * The references panel as it renders in a translated interface: its `aria-label` is the
 * localized `references.referencesPanel.title`, not the English word. The coach mark has to
 * find it anyway — it must not address the panel by copy that the catalogs own.
 */
function mountReferencesPanel(ariaLabel: string, left = 1200) {
  const panel = document.createElement("aside");
  panel.dataset.testid = "references-panel";
  panel.setAttribute("aria-label", ariaLabel);
  panel.getBoundingClientRect = () =>
    ({ top: 0, left, right: left + 320, bottom: 900, width: 320, height: 900 }) as DOMRect;
  document.body.appendChild(panel);
  return panel;
}

describe("GuidanceOverlay", () => {
  it("anchors the sidebar tip beside the sidebar", async () => {
    mountSidebar();
    render(GuidanceOverlay);
    const tip = await screen.findByRole("dialog", { name: "Tip" });
    expect(tip.textContent).toContain("Your outline lives here");
    await waitFor(() => expect(tip.style.left).toBe("320px"));
    expect(tip.classList.contains("is-anchored")).toBe(true);
  });

  it("falls back to the centre when the target can't be measured", async () => {
    render(GuidanceOverlay);
    const tip = await screen.findByRole("dialog", { name: "Tip" });
    expect(tip.classList.contains("is-anchored")).toBe(false);
  });

  it("anchors the references tip beside a panel whose label is translated", async () => {
    // Sidebar and scene panel are already seen, so the overlay advances to references.
    ui.markTooltipSeen("sidebar");
    ui.markTooltipSeen("scenePanel");
    mountReferencesPanel("Ссылки"); // references.referencesPanel.title in ru
    render(GuidanceOverlay);
    const tip = await screen.findByRole("dialog", { name: "Tip" });
    await waitFor(() => expect(tip.classList.contains("is-anchored")).toBe(true));
  });

  it("waits while a modal dialog is open, then shows", async () => {
    mountSidebar();
    const modal = document.createElement("dialog");
    modal.setAttribute("open", "");
    document.body.appendChild(modal);
    render(GuidanceOverlay);
    await waitFor(() => expect(screen.queryByRole("dialog", { name: "Tip" })).toBeNull());

    modal.remove();
    expect(await screen.findByRole("dialog", { name: "Tip" })).toBeTruthy();
  });

  it("dismisses one tip with Got it, or all tips with Disable tips", async () => {
    mountSidebar();
    render(GuidanceOverlay);
    await fireEvent.click(await screen.findByRole("button", { name: /Got it/ }));
    expect(ui.hasSeenTooltip("sidebar")).toBe(true);

    await fireEvent.click(await screen.findByRole("button", { name: /Disable tips/ }));
    expect(ui.guidanceEnabled).toBe(false);
  });
});
