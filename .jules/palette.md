## 2024-07-24 - Bevy Button Hover States
**Learning:** Generic Bevy buttons didn't have hover states. Bevy UI elements can listen for `Changed<Interaction>` to implement this natively.
**Action:** Created a reusable `HoverableButton` component and `button_hover_system` that can be applied to any generic `Button` to manage `BackgroundColor` across `Pressed`, `Hovered`, and `None` states, significantly improving interaction affordance.
