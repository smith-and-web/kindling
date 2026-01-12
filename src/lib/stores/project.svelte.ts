import type { Project, Chapter, Scene, Beat, Character, Location } from "../types";

// Current project state
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

  setCurrentScene(scene: Scene | null) {
    this.currentScene = scene;
    if (!scene) {
      this.beats = [];
    }
  }

  setBeats(beats: Beat[]) {
    this.beats = beats;
  }

  setCharacters(characters: Character[]) {
    this.characters = characters;
  }

  setLocations(locations: Location[]) {
    this.locations = locations;
  }
}

export const currentProject = new ProjectStore();
