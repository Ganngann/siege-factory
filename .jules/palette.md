## 2024-07-23 - Data-Driven Button UX State (Hover & Active)

**Learning:** Data-driven buttons initially lacked hover or pressed states, providing no visual feedback to the user on interaction.
**Action:** Introduced a generic `HoverableButton` component that leverages Bevy's `Interaction` changes to cycle button backgrounds through `theme.btn_inactive`, `theme.btn_hover`, and `theme.btn_active`. Registered globally in `src/ui/mod.rs` so all data-driven buttons inherit accessible interaction feedback.
