# 18 About and feedback

Socket: `npm run qa:visual -- --only 18`. Captures About and the unsubmitted
feedback form in light/dark/narrow. Verify version, links, categories, labels,
required fields and footer controls. The Send button must stay reachable.

Additional acceptance: switch Bug/Feature/Rating and inspect empty/long text,
star selection, validation messages, offline errors and success/reset. Exercise
submission with the component's mocked transport, not the live feedback endpoint.
Do not submit QA feedback or open external links merely to capture this dialog.
Verify Escape/focus return and the Help menu entry separately with native input.
