import { describe, it, expect, beforeEach } from "vitest";
import { currentProject } from "./project.svelte";

describe("currentProject store", () => {
  beforeEach(() => {
    // Reset store state before each test
    currentProject.setProject(null);
    currentProject.setChapters([]);
    currentProject.setScenes([]);
    currentProject.setBeats([]);
  });

  it("should initialize with null project", () => {
    expect(currentProject.value).toBeNull();
  });

  it("should set and get project", () => {
    const mockProject = {
      id: "test-id",
      name: "Test Project",
      source_type: "Plottr" as const,
      source_path: "/path/to/file.pltr",
      created_at: new Date().toISOString(),
      modified_at: new Date().toISOString(),
    };

    currentProject.setProject(mockProject);
    expect(currentProject.value).toEqual(mockProject);
  });

  it("should manage chapters", () => {
    const mockChapters = [
      {
        id: "ch-1",
        project_id: "proj-1",
        title: "Chapter 1",
        position: 0,
      },
    ];

    currentProject.setChapters(mockChapters);
    expect(currentProject.chapters).toEqual(mockChapters);
  });

  it("should manage scenes", () => {
    const mockScenes = [
      {
        id: "sc-1",
        chapter_id: "ch-1",
        title: "Scene 1",
        synopsis: "A scene",
        position: 0,
        prose: null,
      },
    ];

    currentProject.setScenes(mockScenes);
    expect(currentProject.scenes).toEqual(mockScenes);
  });

  it("should set current chapter", () => {
    const mockChapter = {
      id: "ch-1",
      project_id: "proj-1",
      title: "Chapter 1",
      position: 0,
    };

    currentProject.setCurrentChapter(mockChapter);
    expect(currentProject.currentChapter).toEqual(mockChapter);
  });

  it("should set current scene", () => {
    const mockScene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "A scene",
      position: 0,
      prose: null,
    };

    currentProject.setCurrentScene(mockScene);
    expect(currentProject.currentScene).toEqual(mockScene);
  });
});
