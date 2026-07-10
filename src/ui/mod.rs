pub mod components;
pub mod context;
pub mod engine;
pub mod panels;
pub mod registry;
pub mod theme;
pub mod types;

use bevy::prelude::*;

use crate::ui::components::{
    active_toggle::ActiveToggleComponent, button::ButtonComponent,
    conditional_text::ConditionalTextComponent, data_label::DataLabelComponent,
    data_list::DataListComponent, data_text::DataTextComponent,
    h_split::HSplitComponent, hp_bar::HpBarComponent, inventory_grid::InventoryGridComponent,
    label::LabelComponent, progress_bar::ProgressBarComponent,
    recipe_category::RecipeCategoryComponent, recipe_name::RecipeNameComponent,
    recipe_progress::RecipeProgressComponent, section::SectionComponent,
    spacer::SpacerComponent, tier_progress::TierProgressComponent, v_stack::VStackComponent,
};
use crate::ui::engine::LayoutEngine;
use crate::ui::panels::{PanelRegistry, building::BuildingPanelImpl, capsule::CapsulePanelImpl};
use crate::ui::registry::ComponentRegistry;
use crate::ui::theme::Theme;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        let mods = app.world().resource::<crate::core::modding::ModRegistry>().clone();
        app.insert_resource(Theme::load(&mods));

        // Build component registry
        let mut comp_registry = ComponentRegistry::default();
        comp_registry.register(Box::new(LabelComponent));
        comp_registry.register(Box::new(ButtonComponent));
        comp_registry.register(Box::new(DataLabelComponent));
        comp_registry.register(Box::new(SectionComponent));
        comp_registry.register(Box::new(HSplitComponent));
        comp_registry.register(Box::new(VStackComponent));
        comp_registry.register(Box::new(SpacerComponent));
        comp_registry.register(Box::new(ProgressBarComponent));
        comp_registry.register(Box::new(HpBarComponent));
        comp_registry.register(Box::new(InventoryGridComponent));
        comp_registry.register(Box::new(ActiveToggleComponent));
        comp_registry.register(Box::new(RecipeNameComponent));
        comp_registry.register(Box::new(RecipeProgressComponent));
        comp_registry.register(Box::new(TierProgressComponent));
        comp_registry.register(Box::new(RecipeCategoryComponent));
        comp_registry.register(Box::new(DataListComponent));
        comp_registry.register(Box::new(DataTextComponent));
        comp_registry.register(Box::new(ConditionalTextComponent));

        app.insert_resource(crate::ui::components::data_list::DataListSelected::default());

        let theme = app.world().resource::<Theme>().clone();
        let layout_engine = LayoutEngine::new(comp_registry, theme);
        app.insert_resource(layout_engine);

        // Register legacy panels
        let mut panel_registry = PanelRegistry::default();
        panel_registry.register(Box::new(BuildingPanelImpl));
        panel_registry.register(Box::new(CapsulePanelImpl));
        app.insert_resource(panel_registry);
    }
}
