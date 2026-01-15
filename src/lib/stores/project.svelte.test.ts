import { describe, it, expect, beforeEach } from "vitest";
import { currentProject } from "./project.svelte";

describe("currentProject store", () => {
  beforeEach(() => {
    // Reset store state before each test
    currentProject.setProject(null);
    currentProject.setChapters([]);
    currentProject.setScenes([]);
    currentProject.setBeats([]);
    currentProject.setCurrentChapter(null);
    currentProject.setCurrentScene(null);
    currentProject.setCharacters([]);
    currentProject.setLocations([]);
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

  it("should clear all state when setting project to null", () => {
    // Set up some state
    currentProject.setChapters([{ id: "ch-1", project_id: "p1", title: "Ch", position: 0 }]);
    currentProject.setScenes([
      { id: "sc-1", chapter_id: "ch-1", title: "Sc", synopsis: "", position: 0, prose: null },
    ]);
    currentProject.setBeats([
      { id: "b-1", scene_id: "sc-1", content: "Beat", position: 0, prose: null },
    ]);
    currentProject.setCharacters([
      {
        id: "char-1",
        project_id: "p1",
        name: "Hero",
        description: null,
        attributes: {},
        source_id: null,
      },
    ]);
    currentProject.setLocations([
      {
        id: "loc-1",
        project_id: "p1",
        name: "Castle",
        description: null,
        attributes: {},
        source_id: null,
      },
    ]);

    // Clear project
    currentProject.setProject(null);

    expect(currentProject.value).toBeNull();
    expect(currentProject.chapters).toEqual([]);
    expect(currentProject.currentChapter).toBeNull();
    expect(currentProject.currentScene).toBeNull();
    expect(currentProject.scenes).toEqual([]);
    expect(currentProject.beats).toEqual([]);
    expect(currentProject.characters).toEqual([]);
    expect(currentProject.locations).toEqual([]);
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

  it("should add a chapter", () => {
    const chapter1 = { id: "ch-1", project_id: "p1", title: "Chapter 1", position: 0 };
    const chapter2 = { id: "ch-2", project_id: "p1", title: "Chapter 2", position: 1 };

    currentProject.setChapters([chapter1]);
    currentProject.addChapter(chapter2);

    expect(currentProject.chapters).toHaveLength(2);
    expect(currentProject.chapters[1]).toEqual(chapter2);
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

  it("should add a scene", () => {
    const scene1 = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "",
      position: 0,
      prose: null,
    };
    const scene2 = {
      id: "sc-2",
      chapter_id: "ch-1",
      title: "Scene 2",
      synopsis: "",
      position: 1,
      prose: null,
    };

    currentProject.setScenes([scene1]);
    currentProject.addScene(scene2);

    expect(currentProject.scenes).toHaveLength(2);
    expect(currentProject.scenes[1]).toEqual(scene2);
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

  it("should clear scenes and currentScene when setting current chapter to null", () => {
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "",
      position: 0,
      prose: null,
    };
    currentProject.setScenes([scene]);
    currentProject.setCurrentScene(scene);

    currentProject.setCurrentChapter(null);

    expect(currentProject.currentChapter).toBeNull();
    expect(currentProject.scenes).toEqual([]);
    expect(currentProject.currentScene).toBeNull();
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

  it("should clear beats when setting current scene to null", () => {
    const beat = { id: "b-1", scene_id: "sc-1", content: "Beat", position: 0, prose: null };
    currentProject.setBeats([beat]);

    currentProject.setCurrentScene(null);

    expect(currentProject.currentScene).toBeNull();
    expect(currentProject.beats).toEqual([]);
  });

  it("should reorder chapters", () => {
    const chapters = [
      { id: "ch-1", project_id: "p1", title: "Chapter 1", position: 0 },
      { id: "ch-2", project_id: "p1", title: "Chapter 2", position: 1 },
      { id: "ch-3", project_id: "p1", title: "Chapter 3", position: 2 },
    ];
    currentProject.setChapters(chapters);

    // Reorder: move ch-3 to first position
    currentProject.reorderChapters(["ch-3", "ch-1", "ch-2"]);

    expect(currentProject.chapters[0].id).toBe("ch-3");
    expect(currentProject.chapters[0].position).toBe(0);
    expect(currentProject.chapters[1].id).toBe("ch-1");
    expect(currentProject.chapters[1].position).toBe(1);
    expect(currentProject.chapters[2].id).toBe("ch-2");
    expect(currentProject.chapters[2].position).toBe(2);
  });

  it("should handle reorder with invalid chapter ids", () => {
    const chapters = [
      { id: "ch-1", project_id: "p1", title: "Chapter 1", position: 0 },
      { id: "ch-2", project_id: "p1", title: "Chapter 2", position: 1 },
    ];
    currentProject.setChapters(chapters);

    // Include an invalid ID - should be filtered out
    currentProject.reorderChapters(["ch-2", "invalid-id", "ch-1"]);

    expect(currentProject.chapters).toHaveLength(2);
    expect(currentProject.chapters[0].id).toBe("ch-2");
    expect(currentProject.chapters[1].id).toBe("ch-1");
  });

  it("should reorder scenes", () => {
    const scenes = [
      { id: "sc-1", chapter_id: "ch-1", title: "Scene 1", synopsis: "", position: 0, prose: null },
      { id: "sc-2", chapter_id: "ch-1", title: "Scene 2", synopsis: "", position: 1, prose: null },
      { id: "sc-3", chapter_id: "ch-1", title: "Scene 3", synopsis: "", position: 2, prose: null },
    ];
    currentProject.setScenes(scenes);

    // Reorder: reverse order
    currentProject.reorderScenes(["sc-3", "sc-2", "sc-1"]);

    expect(currentProject.scenes[0].id).toBe("sc-3");
    expect(currentProject.scenes[0].position).toBe(0);
    expect(currentProject.scenes[1].id).toBe("sc-2");
    expect(currentProject.scenes[1].position).toBe(1);
    expect(currentProject.scenes[2].id).toBe("sc-1");
    expect(currentProject.scenes[2].position).toBe(2);
  });

  it("should remove a chapter", () => {
    const chapters = [
      { id: "ch-1", project_id: "p1", title: "Chapter 1", position: 0 },
      { id: "ch-2", project_id: "p1", title: "Chapter 2", position: 1 },
    ];
    currentProject.setChapters(chapters);

    currentProject.removeChapter("ch-1");

    expect(currentProject.chapters).toHaveLength(1);
    expect(currentProject.chapters[0].id).toBe("ch-2");
  });

  it("should clear related state when removing the current chapter", () => {
    const chapter = { id: "ch-1", project_id: "p1", title: "Chapter 1", position: 0 };
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "",
      position: 0,
      prose: null,
    };
    const beat = { id: "b-1", scene_id: "sc-1", content: "Beat", position: 0, prose: null };

    currentProject.setChapters([chapter]);
    currentProject.setCurrentChapter(chapter);
    currentProject.setScenes([scene]);
    currentProject.setCurrentScene(scene);
    currentProject.setBeats([beat]);

    currentProject.removeChapter("ch-1");

    expect(currentProject.chapters).toEqual([]);
    expect(currentProject.currentChapter).toBeNull();
    expect(currentProject.scenes).toEqual([]);
    expect(currentProject.currentScene).toBeNull();
    expect(currentProject.beats).toEqual([]);
  });

  it("should not clear state when removing a different chapter", () => {
    const chapter1 = { id: "ch-1", project_id: "p1", title: "Chapter 1", position: 0 };
    const chapter2 = { id: "ch-2", project_id: "p1", title: "Chapter 2", position: 1 };

    currentProject.setChapters([chapter1, chapter2]);
    currentProject.setCurrentChapter(chapter1);

    currentProject.removeChapter("ch-2");

    expect(currentProject.currentChapter).toEqual(chapter1);
  });

  it("should remove a scene", () => {
    const scenes = [
      { id: "sc-1", chapter_id: "ch-1", title: "Scene 1", synopsis: "", position: 0, prose: null },
      { id: "sc-2", chapter_id: "ch-1", title: "Scene 2", synopsis: "", position: 1, prose: null },
    ];
    currentProject.setScenes(scenes);

    currentProject.removeScene("sc-1");

    expect(currentProject.scenes).toHaveLength(1);
    expect(currentProject.scenes[0].id).toBe("sc-2");
  });

  it("should clear beats when removing the current scene", () => {
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "",
      position: 0,
      prose: null,
    };
    const beat = { id: "b-1", scene_id: "sc-1", content: "Beat", position: 0, prose: null };

    currentProject.setScenes([scene]);
    currentProject.setCurrentScene(scene);
    currentProject.setBeats([beat]);

    currentProject.removeScene("sc-1");

    expect(currentProject.scenes).toEqual([]);
    expect(currentProject.currentScene).toBeNull();
    expect(currentProject.beats).toEqual([]);
  });

  it("should manage beats", () => {
    const beats = [
      { id: "b-1", scene_id: "sc-1", content: "Beat 1", position: 0, prose: null },
      { id: "b-2", scene_id: "sc-1", content: "Beat 2", position: 1, prose: null },
    ];

    currentProject.setBeats(beats);
    expect(currentProject.beats).toEqual(beats);
  });

  it("should update beat prose", () => {
    const beats = [
      { id: "b-1", scene_id: "sc-1", content: "Beat 1", position: 0, prose: null },
      { id: "b-2", scene_id: "sc-1", content: "Beat 2", position: 1, prose: null },
    ];
    currentProject.setBeats(beats);

    currentProject.updateBeatProse("b-1", "This is the prose for beat 1");

    expect(currentProject.beats[0].prose).toBe("This is the prose for beat 1");
    expect(currentProject.beats[1].prose).toBeNull();
  });

  it("should not modify beats when updating prose for non-existent beat", () => {
    const beats = [{ id: "b-1", scene_id: "sc-1", content: "Beat 1", position: 0, prose: null }];
    currentProject.setBeats(beats);

    currentProject.updateBeatProse("non-existent", "Some prose");

    expect(currentProject.beats[0].prose).toBeNull();
  });

  it("should manage characters", () => {
    const characters = [
      {
        id: "char-1",
        project_id: "p1",
        name: "Hero",
        description: null,
        attributes: {},
        source_id: null,
      },
      {
        id: "char-2",
        project_id: "p1",
        name: "Villain",
        description: null,
        attributes: {},
        source_id: null,
      },
    ];

    currentProject.setCharacters(characters);
    expect(currentProject.characters).toEqual(characters);
  });

  it("should manage locations", () => {
    const locations = [
      {
        id: "loc-1",
        project_id: "p1",
        name: "Castle",
        description: null,
        attributes: {},
        source_id: null,
      },
      {
        id: "loc-2",
        project_id: "p1",
        name: "Forest",
        description: null,
        attributes: {},
        source_id: null,
      },
    ];

    currentProject.setLocations(locations);
    expect(currentProject.locations).toEqual(locations);
  });
});
