import { describe, it, expect, beforeEach } from "vitest";
import { currentProject } from "./project.svelte";
import type { Project, ReferenceTypeId } from "../types";

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
    const mockProject: Project = {
      id: "test-id",
      name: "Test Project",
      source_type: "Plottr" as const,
      source_path: "/path/to/file.pltr",
      created_at: new Date().toISOString(),
      modified_at: new Date().toISOString(),
      author_pen_name: null,
      genre: null,
      description: null,
      word_target: null,
      reference_types: ["characters", "locations"] as ReferenceTypeId[],
    };

    currentProject.setProject(mockProject);
    expect(currentProject.value).toEqual(mockProject);
  });

  it("should clear all state when setting project to null", () => {
    // Set up some state
    currentProject.setChapters([
      {
        id: "ch-1",
        project_id: "p1",
        title: "Ch",
        position: 0,
        archived: false,
        locked: false,
        is_part: false,
      },
    ]);
    currentProject.setScenes([
      {
        id: "sc-1",
        chapter_id: "ch-1",
        title: "Sc",
        synopsis: "",
        position: 0,
        prose: null,
        archived: false,
        locked: false,
      },
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
        archived: false,
        locked: false,
        is_part: false,
      },
    ];

    currentProject.setChapters(mockChapters);
    expect(currentProject.chapters).toEqual(mockChapters);
  });

  it("should add a chapter", () => {
    const chapter1 = {
      id: "ch-1",
      project_id: "p1",
      title: "Chapter 1",
      position: 0,
      archived: false,
      locked: false,
      is_part: false,
    };
    const chapter2 = {
      id: "ch-2",
      project_id: "p1",
      title: "Chapter 2",
      position: 1,
      archived: false,
      locked: false,
      is_part: false,
    };

    currentProject.setChapters([chapter1]);
    currentProject.addChapter(chapter2);

    expect(currentProject.chapters).toHaveLength(2);
    expect(currentProject.chapters[1]).toEqual(chapter2);
  });

  it("should add a chapter after a specific chapter using afterId", () => {
    const chapter1 = {
      id: "ch-1",
      project_id: "p1",
      title: "Chapter 1",
      position: 0,
      archived: false,
      locked: false,
      is_part: false,
    };
    const chapter2 = {
      id: "ch-2",
      project_id: "p1",
      title: "Chapter 2",
      position: 1,
      archived: false,
      locked: false,
      is_part: false,
    };
    const chapter3 = {
      id: "ch-3",
      project_id: "p1",
      title: "Chapter 3",
      position: 2,
      archived: false,
      locked: false,
      is_part: false,
    };

    currentProject.setChapters([chapter1, chapter3]);
    // Insert chapter2 after chapter1
    currentProject.addChapter(chapter2, "ch-1");

    expect(currentProject.chapters).toHaveLength(3);
    expect(currentProject.chapters[0].id).toBe("ch-1");
    expect(currentProject.chapters[1].id).toBe("ch-2");
    expect(currentProject.chapters[2].id).toBe("ch-3");
  });

  it("should append to end if afterId is not found", () => {
    const chapter1 = {
      id: "ch-1",
      project_id: "p1",
      title: "Chapter 1",
      position: 0,
      archived: false,
      locked: false,
      is_part: false,
    };
    const chapter2 = {
      id: "ch-2",
      project_id: "p1",
      title: "Chapter 2",
      position: 1,
      archived: false,
      locked: false,
      is_part: false,
    };

    currentProject.setChapters([chapter1]);
    // Try to insert after non-existent chapter
    currentProject.addChapter(chapter2, "non-existent");

    expect(currentProject.chapters).toHaveLength(2);
    expect(currentProject.chapters[0].id).toBe("ch-1");
    expect(currentProject.chapters[1].id).toBe("ch-2");
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
        archived: false,
        locked: false,
        is_part: false,
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
      archived: false,
      locked: false,
      is_part: false,
    };
    const scene2 = {
      id: "sc-2",
      chapter_id: "ch-1",
      title: "Scene 2",
      synopsis: "",
      position: 1,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
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
      archived: false,
      locked: false,
      is_part: false,
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
      archived: false,
      locked: false,
      is_part: false,
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
      archived: false,
      locked: false,
      is_part: false,
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
      {
        id: "ch-1",
        project_id: "p1",
        title: "Chapter 1",
        position: 0,
        archived: false,
        locked: false,
        is_part: false,
      },
      {
        id: "ch-2",
        project_id: "p1",
        title: "Chapter 2",
        position: 1,
        archived: false,
        locked: false,
        is_part: false,
      },
      {
        id: "ch-3",
        project_id: "p1",
        title: "Chapter 3",
        position: 2,
        archived: false,
        locked: false,
        is_part: false,
      },
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
      {
        id: "ch-1",
        project_id: "p1",
        title: "Chapter 1",
        position: 0,
        archived: false,
        locked: false,
        is_part: false,
      },
      {
        id: "ch-2",
        project_id: "p1",
        title: "Chapter 2",
        position: 1,
        archived: false,
        locked: false,
        is_part: false,
      },
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
      {
        id: "sc-1",
        chapter_id: "ch-1",
        title: "Scene 1",
        synopsis: "",
        position: 0,
        prose: null,
        archived: false,
        locked: false,
        is_part: false,
      },
      {
        id: "sc-2",
        chapter_id: "ch-1",
        title: "Scene 2",
        synopsis: "",
        position: 1,
        prose: null,
        archived: false,
        locked: false,
        is_part: false,
      },
      {
        id: "sc-3",
        chapter_id: "ch-1",
        title: "Scene 3",
        synopsis: "",
        position: 2,
        prose: null,
        archived: false,
        locked: false,
        is_part: false,
      },
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
      {
        id: "ch-1",
        project_id: "p1",
        title: "Chapter 1",
        position: 0,
        archived: false,
        locked: false,
        is_part: false,
      },
      {
        id: "ch-2",
        project_id: "p1",
        title: "Chapter 2",
        position: 1,
        archived: false,
        locked: false,
        is_part: false,
      },
    ];
    currentProject.setChapters(chapters);

    currentProject.removeChapter("ch-1");

    expect(currentProject.chapters).toHaveLength(1);
    expect(currentProject.chapters[0].id).toBe("ch-2");
  });

  it("should clear related state when removing the current chapter", () => {
    const chapter = {
      id: "ch-1",
      project_id: "p1",
      title: "Chapter 1",
      position: 0,
      archived: false,
      locked: false,
      is_part: false,
    };
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "",
      position: 0,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
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
    const chapter1 = {
      id: "ch-1",
      project_id: "p1",
      title: "Chapter 1",
      position: 0,
      archived: false,
      locked: false,
      is_part: false,
    };
    const chapter2 = {
      id: "ch-2",
      project_id: "p1",
      title: "Chapter 2",
      position: 1,
      archived: false,
      locked: false,
      is_part: false,
    };

    currentProject.setChapters([chapter1, chapter2]);
    currentProject.setCurrentChapter(chapter1);

    currentProject.removeChapter("ch-2");

    expect(currentProject.currentChapter).toEqual(chapter1);
  });

  it("should remove a scene", () => {
    const scenes = [
      {
        id: "sc-1",
        chapter_id: "ch-1",
        title: "Scene 1",
        synopsis: "",
        position: 0,
        prose: null,
        archived: false,
        locked: false,
        is_part: false,
      },
      {
        id: "sc-2",
        chapter_id: "ch-1",
        title: "Scene 2",
        synopsis: "",
        position: 1,
        prose: null,
        archived: false,
        locked: false,
        is_part: false,
      },
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
      archived: false,
      locked: false,
      is_part: false,
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

  it("should add a beat", () => {
    const beat1 = { id: "b-1", scene_id: "sc-1", content: "Beat 1", position: 0, prose: null };
    const beat2 = { id: "b-2", scene_id: "sc-1", content: "Beat 2", position: 1, prose: null };

    currentProject.setBeats([beat1]);
    currentProject.addBeat(beat2);

    expect(currentProject.beats).toHaveLength(2);
    expect(currentProject.beats[1]).toEqual(beat2);
  });

  it("should update scene synopsis", () => {
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "Original synopsis",
      position: 0,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setScenes([scene]);

    currentProject.updateSceneSynopsis("sc-1", "Updated synopsis");

    expect(currentProject.scenes[0].synopsis).toBe("Updated synopsis");
  });

  it("should update scene synopsis and currentScene when it matches", () => {
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "Original synopsis",
      position: 0,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setScenes([scene]);
    currentProject.setCurrentScene(scene);

    currentProject.updateSceneSynopsis("sc-1", "Updated synopsis");

    expect(currentProject.scenes[0].synopsis).toBe("Updated synopsis");
    expect(currentProject.currentScene?.synopsis).toBe("Updated synopsis");
  });

  it("should update scene synopsis to null", () => {
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "Original synopsis",
      position: 0,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setScenes([scene]);

    currentProject.updateSceneSynopsis("sc-1", null);

    expect(currentProject.scenes[0].synopsis).toBeNull();
  });

  it("should update chapter", () => {
    const chapter = {
      id: "ch-1",
      project_id: "p1",
      title: "Original Title",
      position: 0,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setChapters([chapter]);

    currentProject.updateChapter("ch-1", { title: "Updated Title", locked: true });

    expect(currentProject.chapters[0].title).toBe("Updated Title");
    expect(currentProject.chapters[0].locked).toBe(true);
    expect(currentProject.chapters[0].position).toBe(0); // unchanged
  });

  it("should update chapter and currentChapter when it matches", () => {
    const chapter = {
      id: "ch-1",
      project_id: "p1",
      title: "Original Title",
      position: 0,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setChapters([chapter]);
    currentProject.setCurrentChapter(chapter);

    currentProject.updateChapter("ch-1", { title: "Updated Title" });

    expect(currentProject.chapters[0].title).toBe("Updated Title");
    expect(currentProject.currentChapter?.title).toBe("Updated Title");
  });

  it("should not update currentChapter when updating a different chapter", () => {
    const chapter1 = {
      id: "ch-1",
      project_id: "p1",
      title: "Chapter 1",
      position: 0,
      archived: false,
      locked: false,
      is_part: false,
    };
    const chapter2 = {
      id: "ch-2",
      project_id: "p1",
      title: "Chapter 2",
      position: 1,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setChapters([chapter1, chapter2]);
    currentProject.setCurrentChapter(chapter1);

    currentProject.updateChapter("ch-2", { title: "Updated Chapter 2" });

    expect(currentProject.currentChapter?.title).toBe("Chapter 1");
    expect(currentProject.chapters[1].title).toBe("Updated Chapter 2");
  });

  it("should update scene", () => {
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Original Title",
      synopsis: "Synopsis",
      position: 0,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setScenes([scene]);

    currentProject.updateScene("sc-1", { title: "Updated Title", locked: true });

    expect(currentProject.scenes[0].title).toBe("Updated Title");
    expect(currentProject.scenes[0].locked).toBe(true);
    expect(currentProject.scenes[0].synopsis).toBe("Synopsis"); // unchanged
  });

  it("should update scene and currentScene when it matches", () => {
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Original Title",
      synopsis: "Synopsis",
      position: 0,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setScenes([scene]);
    currentProject.setCurrentScene(scene);

    currentProject.updateScene("sc-1", { title: "Updated Title" });

    expect(currentProject.scenes[0].title).toBe("Updated Title");
    expect(currentProject.currentScene?.title).toBe("Updated Title");
  });

  it("should not update currentScene when updating a different scene", () => {
    const scene1 = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Scene 1",
      synopsis: "",
      position: 0,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
    };
    const scene2 = {
      id: "sc-2",
      chapter_id: "ch-1",
      title: "Scene 2",
      synopsis: "",
      position: 1,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setScenes([scene1, scene2]);
    currentProject.setCurrentScene(scene1);

    currentProject.updateScene("sc-2", { title: "Updated Scene 2" });

    expect(currentProject.currentScene?.title).toBe("Scene 1");
    expect(currentProject.scenes[1].title).toBe("Updated Scene 2");
  });

  it("should refresh current scene", () => {
    const scene = {
      id: "sc-1",
      chapter_id: "ch-1",
      title: "Original Title",
      synopsis: "Synopsis",
      position: 0,
      prose: null,
      archived: false,
      locked: false,
      is_part: false,
    };
    currentProject.setScenes([scene]);
    currentProject.setCurrentScene(scene);

    const updatedScene = {
      ...scene,
      title: "Refreshed Title",
      prose: "Some prose",
    };
    currentProject.refreshCurrentScene(updatedScene);

    expect(currentProject.currentScene?.title).toBe("Refreshed Title");
    expect(currentProject.currentScene?.prose).toBe("Some prose");
    expect(currentProject.scenes[0].title).toBe("Refreshed Title");
    expect(currentProject.scenes[0].prose).toBe("Some prose");
  });
});
