import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import type { InvokeArgs } from "@tauri-apps/api/core";
import ReferenceClassificationDialog from "./ReferenceClassificationDialog.svelte";
import type { Character, Location, Project, ReferenceItem } from "../types";

const invokeMock = vi.mocked(invoke);

const mockProject: Project = {
  id: "project-1",
  name: "Test Project",
  source_type: "Plottr",
  source_path: null,
  created_at: "2024-01-01T00:00:00.000Z",
  modified_at: "2024-01-01T00:00:00.000Z",
  author_pen_name: null,
  genre: null,
  description: null,
  word_target: null,
  reference_types: ["characters", "locations", "items", "objectives", "organizations"],
  project_type: "novel",
  target_page_count: null,
};

const mockCharacters: Character[] = [
  {
    id: "char-1",
    project_id: "project-1",
    name: "Alice",
    description: "<p>Lead character</p>",
    attributes: {},
    source_id: null,
  },
];

const mockLocations: Location[] = [
  {
    id: "loc-1",
    project_id: "project-1",
    name: "HQ",
    description: null,
    attributes: {},
    source_id: null,
  },
];

const mockItems: ReferenceItem[] = [
  {
    id: "item-1",
    project_id: "project-1",
    reference_type: "items",
    name: "Widget",
    description: null,
    attributes: {},
    source_id: null,
  },
];

const mockObjectives: ReferenceItem[] = [
  {
    id: "objective-1",
    project_id: "project-1",
    reference_type: "objectives",
    name: "Escape",
    description: null,
    attributes: {},
    source_id: null,
  },
];

const mockOrganizations: ReferenceItem[] = [
  {
    id: "org-1",
    project_id: "project-1",
    reference_type: "organizations",
    name: "Consortium",
    description: null,
    attributes: {},
    source_id: null,
  },
];

function setupInvokeMocks() {
  invokeMock.mockImplementation((command: string, args?: InvokeArgs) => {
    if (command === "get_characters") {
      return Promise.resolve(mockCharacters);
    }
    if (command === "get_locations") {
      return Promise.resolve(mockLocations);
    }
    if (command === "get_references") {
      const referenceType =
        args && !Array.isArray(args) ? (args as { referenceType?: unknown }).referenceType : null;
      if (referenceType === "items") {
        return Promise.resolve(mockItems);
      }
      if (referenceType === "objectives") {
        return Promise.resolve(mockObjectives);
      }
      if (referenceType === "organizations") {
        return Promise.resolve(mockOrganizations);
      }
      return Promise.resolve([]);
    }
    if (command === "reclassify_references") {
      return Promise.resolve(mockProject);
    }
    return Promise.reject(new Error(`Unexpected invoke: ${command}`));
  });
}

describe("ReferenceClassificationDialog", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads and displays references", async () => {
    setupInvokeMocks();
    render(ReferenceClassificationDialog, {
      props: {
        projectId: "project-1",
        onClose: vi.fn(),
        onComplete: vi.fn(),
      },
    });

    expect(screen.getByText("Loading references…")).toBeTruthy();

    expect(await screen.findByText("Alice")).toBeTruthy();
    expect(screen.getByText("HQ")).toBeTruthy();
    expect(screen.getByText("Widget")).toBeTruthy();
    expect(screen.getByText("Escape")).toBeTruthy();
    expect(screen.getByText("Consortium")).toBeTruthy();
  });

  it("skips without changes", async () => {
    setupInvokeMocks();
    const onClose = vi.fn();
    render(ReferenceClassificationDialog, {
      props: {
        projectId: "project-1",
        onClose,
        onComplete: vi.fn(),
      },
    });

    await screen.findByText("Alice");

    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(invokeMock).not.toHaveBeenCalledWith("reclassify_references", expect.anything());
  });

  it("reclassifies when a type changes", async () => {
    setupInvokeMocks();
    const onComplete = vi.fn();
    render(ReferenceClassificationDialog, {
      props: {
        projectId: "project-1",
        onClose: vi.fn(),
        onComplete,
      },
    });

    await screen.findByText("Alice");

    const selects = screen.getAllByRole("combobox");
    await fireEvent.change(selects[0], { target: { value: "locations" } });

    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("reclassify_references", {
        projectId: "project-1",
        changes: [{ reference_id: "char-1", new_type: "locations" }],
      });
    });

    await waitFor(() => {
      expect(onComplete).toHaveBeenCalledWith(mockProject);
    });
  });

  it("keeps the table visible and shows the save-failure notice when saving fails", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    setupInvokeMocks();
    const fallback = invokeMock.getMockImplementation()!;
    invokeMock.mockImplementation((command: string, args?: InvokeArgs) =>
      command === "reclassify_references"
        ? Promise.reject(new Error("Database is locked"))
        : fallback(command, args)
    );
    const onComplete = vi.fn();
    const onClose = vi.fn();
    render(ReferenceClassificationDialog, {
      props: { projectId: "project-1", onClose, onComplete },
    });

    await screen.findByText("Alice");
    const select = screen.getByRole("combobox", { name: "Type for Alice" }) as HTMLSelectElement;
    await fireEvent.change(select, { target: { value: "locations" } });
    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Couldn’t save reference types");
    expect(alert.textContent).toContain("Database is locked");
    expect(alert.textContent).not.toContain("Couldn’t load references");

    // The table and the writer's pending choice survive the failure so they can retry.
    expect(screen.getByRole("table")).toBeTruthy();
    expect(screen.getByText("Consortium")).toBeTruthy();
    expect(select.value).toBe("locations");
    expect(select.disabled).toBe(false);
    expect(
      (screen.getByRole("button", { name: "Apply changes" }) as HTMLButtonElement).disabled
    ).toBe(false);
    expect(onComplete).not.toHaveBeenCalled();
    expect(onClose).not.toHaveBeenCalled();
    vi.restoreAllMocks();
  });

  it("uses a fallback save-failure message for a non-Error rejection", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    setupInvokeMocks();
    const fallback = invokeMock.getMockImplementation()!;
    invokeMock.mockImplementation((command: string, args?: InvokeArgs) =>
      command === "reclassify_references" ? Promise.reject("locked") : fallback(command, args)
    );
    render(ReferenceClassificationDialog, {
      props: { projectId: "project-1", onClose: vi.fn(), onComplete: vi.fn() },
    });

    await screen.findByText("Alice");
    await fireEvent.change(screen.getByRole("combobox", { name: "Type for HQ" }), {
      target: { value: "items" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Apply changes" }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Couldn’t save reference types");
    expect(alert.textContent).toContain("Failed to save reference classifications");
    expect(screen.getByRole("table")).toBeTruthy();
    vi.restoreAllMocks();
  });

  it("renders an imported description as inert formatted prose", async () => {
    setupInvokeMocks();
    const description =
      '<p><strong>Bio:</strong> Kind<meta http-equiv="refresh" content="0;url=https://example.com"><form action="https://example.com"><input name="password"><button>Sign in</button></form><a href="https://example.com" style="position:fixed;inset:0">Continue</a> and <em>brave</em></p>';
    const fallback = invokeMock.getMockImplementation()!;
    invokeMock.mockImplementation((command: string, args?: InvokeArgs) =>
      command === "get_characters"
        ? Promise.resolve([{ ...mockCharacters[0], description }])
        : fallback(command, args)
    );
    const { container } = render(ReferenceClassificationDialog, {
      props: { projectId: "project-1", onClose: vi.fn(), onComplete: vi.fn() },
    });
    await screen.findByText("Alice");
    const prose = container.querySelector(".classify-desc")!;
    expect(prose.querySelector("meta, form, input, button, a, [style], [href]")).toBeNull();
    expect(prose.querySelector("strong")?.textContent).toBe("Bio:");
    expect(prose.querySelector("em")?.textContent).toBe("brave");
  });
});
