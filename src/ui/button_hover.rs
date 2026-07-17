use bevy::prelude::*;
use crate::ui::theme::Theme;

pub fn button_hover_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    theme: Res<Theme>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                color.0 = theme.btn_hover;
            }
            Interaction::None => {
                color.0 = theme.btn_inactive;
            }
            Interaction::Pressed => {
                color.0 = theme.btn_active;
            }
        }
    }
}
