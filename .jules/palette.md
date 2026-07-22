## 2024-05-23 - Interactive Feedback for Generic Buttons
**Learning:** Generic buttons lacked dynamic visual feedback (hover and pressed states), reducing perceived interactability.
**Action:** Always include a `HoverableButton` component (with `inactive`, `hover`, and `pressed` states from the theme) alongside the `Button` component, and ensure a system like `button_hover_system` handles `Changed<Interaction>` to update the `BackgroundColor` dynamically.
