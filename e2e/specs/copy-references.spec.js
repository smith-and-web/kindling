import { waitForAppReady, skipOnboardingIfPresent, waitForEditor } from "./helpers.js";
async function invoke(command, args = {}) {
  const response = await browser.executeAsync(async (command, args, done) => {
    try { done({value: await window.__KINDLING_TEST__.invoke(command, args)}); }
    catch (error) { done({error:JSON.stringify(error)}); }
  }, command, args);
  if (response.error) throw new Error(response.error);
  return response.value;
}
async function openProject(name) {
  const close = await $('[aria-label="Close project"]');
  if (await close.isExisting()) await close.click();
  await $('[data-testid="recent-projects"]').waitForDisplayed();
  const cards = await $$('[data-testid="project-card"]');
  for (const card of cards) if ((await card.getText()).includes(name)) { await card.click(); await waitForEditor(); return; }
  throw new Error(`Project not listed: ${name}`);
}
describe("independent reference transfers", () => {
  const projects = [];
  after(async () => {
    const close = await $('[aria-label="Close project"]');
    if (await close.isExisting()) await close.click();
    for (const project of projects) await invoke("delete_project", {projectId:project.id});
  });
  it("copies through native IPC, persists, links and edits independently", async () => {
    await waitForAppReady(); await skipOnboardingIfPresent();
    const source = await invoke("create_blank_project", {name:`Copy source ${Date.now()}`}); projects.push(source);
    const destination = await invoke("create_blank_project", {name:`Copy destination ${Date.now()}`}); projects.push(destination);
    const reference = await invoke("create_reference", {projectId:source.id,referenceType:"characters",reference:{name:"Mara",description:"Captain",attributes:{notes:"Keep the crew safe"}}});
    const field = await invoke("create_field_definition", {projectId:source.id,definition:{entity_type:"character",name:"Rank",field_type:"text"}});
    await invoke("set_field_value", {fieldDefinitionId:field.id,entityId:reference,value:"Captain"});
    const tag = await invoke("create_tag", {projectId:source.id,name:"Crew",color:null,parentId:null});
    await invoke("tag_entity", {tagId:tag.id,entityType:"character",entityId:reference});
    await browser.refresh(); await waitForAppReady(); await skipOnboardingIfPresent();
    await openProject(destination.name);
    await $('[aria-label="Copy references from project…"]').click();
    await $('#copy-source').selectByAttribute("value", source.id);
    const copy = await $('button=Copy 1 references'); await copy.waitForEnabled(); await copy.click();
    await $('button=Done').waitForDisplayed(); await $('button=Done').click();
    const copied = (await invoke("get_references", {projectId:destination.id,referenceType:"characters"}))[0];
    expect(copied.id).not.toBe(reference); expect(copied.source_id).toBeNull();
    expect(copied.attributes.notes).toBe("Keep the crew safe");
    expect((await invoke("get_field_values", {entityId:copied.id}))[0].value).toBe("Captain");
    expect((await invoke("get_entity_tags", {entityType:"character",entityId:copied.id}))[0].name).toBe("Crew");
    await invoke("update_reference", {referenceId:copied.id,referenceType:"characters",reference:{name:"Mara",description:"Admiral",attributes:copied.attributes}});
    const chapter = (await invoke("get_chapters", {projectId:destination.id}))[0];
    const scene = (await invoke("get_scenes", {chapterId:chapter.id}))[0];
    await invoke("save_scene_reference_state", {sceneId:scene.id,referenceType:"characters",states:[{reference_id:copied.id,position:0,expanded:true}]});
    expect((await invoke("get_references", {projectId:source.id,referenceType:"characters"}))[0].description).toBe("Captain");
    // A new WebDriver session restarts the app against the same local database.
    await browser.reloadSession(); await waitForAppReady(); await skipOnboardingIfPresent();
    await openProject(destination.name);
    expect((await invoke("get_references", {projectId:destination.id,referenceType:"characters"}))[0].description).toBe("Admiral");
    expect((await invoke("get_scene_reference_state", {sceneId:scene.id}))[0].reference_id).toBe(copied.id);
    await $('[aria-label="Copy references from project…"]').click();
    await $('#copy-source').selectByAttribute("value", source.id);
    await browser.waitUntil(async () => (await $('dialog').getText()).includes("1 possible duplicates skipped"));
    expect(await $('button=Copy 0 references').isEnabled()).toBe(false);
    await $('button=Cancel').click();
  });
});
