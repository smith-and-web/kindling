/**
 * Project Store - Manages all project-related state
 *
 * This store holds the currently loaded project data including chapters, scenes,
 * beats, characters, and locations. It uses Svelte 5 runes ($state) for reactivity.
 *
 * Usage:
 *   import { currentProject } from "$lib/stores/project.svelte";
 *   currentProject.setProject(project);
 *   console.log(currentProject.chapters);
 *
 * The store maintains the current selection state (currentChapter, currentScene)
 * and automatically clears child data when selections change.
 *
 * @see types.ts for the shape of each data type
 * @see ui.svelte.ts for UI-related state (panel visibility, onboarding, etc.)
 */
import type { Project, Chapter, Scene, Beat, Character, Location } from "../types";

class ProjectStore {
  value = $state<Project | null>(null);
  chapters = $state<Chapter[]>([]);
  currentChapter = $state<Chapter | null>(null);
  currentScene = $state<Scene | null>(null);
  scenes = $state<Scene[]>([]);
  beats = $state<Beat[]>([]);
  characters = $state<Character[]>([]);
  locations = $state<Location[]>([]);

  setProject(project: Project | null) {
    this.value = project;
    if (!project) {
      this.chapters = [];
      this.currentChapter = null;
      this.currentScene = null;
      this.scenes = [];
      this.beats = [];
      this.characters = [];
      this.locations = [];
    }
  }

  setChapters(chapters: Chapter[]) {
    this.chapters = chapters;
  }

  addChapter(chapter: Chapter) {
    this.chapters = [...this.chapters, chapter];
  }

  setCurrentChapter(chapter: Chapter | null) {
    this.currentChapter = chapter;
    if (!chapter) {
      this.scenes = [];
      this.currentScene = null;
    }
  }

  setScenes(scenes: Scene[]) {
    this.scenes = scenes;
  }

  addScene(scene: Scene) {
    this.scenes = [...this.scenes, scene];
  }

  reorderChapters(chapterIds: string[]) {
    const chapterMap = new Map(this.chapters.map((c) => [c.id, c]));
    this.chapters = chapterIds
      .map((id) => chapterMap.get(id))
      .filter((c): c is Chapter => c !== undefined)
      .map((c, i) => ({ ...c, position: i }));
  }

  reorderScenes(sceneIds: string[]) {
    const sceneMap = new Map(this.scenes.map((s) => [s.id, s]));
    this.scenes = sceneIds
      .map((id) => sceneMap.get(id))
      .filter((s): s is Scene => s !== undefined)
      .map((s, i) => ({ ...s, position: i }));
  }

  removeChapter(chapterId: string) {
    this.chapters = this.chapters.filter((c) => c.id !== chapterId);
    if (this.currentChapter?.id === chapterId) {
      this.currentChapter = null;
      this.scenes = [];
      this.currentScene = null;
      this.beats = [];
    }
  }

  removeScene(sceneId: string) {
    this.scenes = this.scenes.filter((s) => s.id !== sceneId);
    if (this.currentScene?.id === sceneId) {
      this.currentScene = null;
      this.beats = [];
    }
  }

  setCurrentScene(scene: Scene | null) {
    this.currentScene = scene;
    if (!scene) {
      this.beats = [];
    }
  }

  setBeats(beats: Beat[]) {
    this.beats = beats;
  }

  updateBeatProse(beatId: string, prose: string) {
    this.beats = this.beats.map((beat) => (beat.id === beatId ? { ...beat, prose } : beat));
  }

  addBeat(beat: Beat) {
    this.beats = [...this.beats, beat];
  }

  updateSceneSynopsis(sceneId: string, synopsis: string | null) {
    this.scenes = this.scenes.map((scene) =>
      scene.id === sceneId ? { ...scene, synopsis } : scene
    );
    if (this.currentScene?.id === sceneId) {
      this.currentScene = { ...this.currentScene, synopsis };
    }
  }

  setCharacters(characters: Character[]) {
    this.characters = characters;
  }

  setLocations(locations: Location[]) {
    this.locations = locations;
  }

  updateChapter(chapterId: string, updates: Partial<Chapter>) {
    this.chapters = this.chapters.map((chapter) =>
      chapter.id === chapterId ? { ...chapter, ...updates } : chapter
    );
    if (this.currentChapter?.id === chapterId) {
      this.currentChapter = { ...this.currentChapter, ...updates };
    }
  }

  updateScene(sceneId: string, updates: Partial<Scene>) {
    this.scenes = this.scenes.map((scene) =>
      scene.id === sceneId ? { ...scene, ...updates } : scene
    );
    if (this.currentScene?.id === sceneId) {
      this.currentScene = { ...this.currentScene, ...updates };
    }
  }

  refreshCurrentScene(scene: Scene) {
    this.currentScene = scene;
    // Also update the scene in the scenes array
    this.scenes = this.scenes.map((s) => (s.id === scene.id ? scene : s));
  }
}

export const currentProject = new ProjectStore();
