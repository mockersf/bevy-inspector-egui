use super::nalgebra_conversions::*;
use crate::egui::Grid;
use crate::impls::NumberAttributes;
use crate::{utils, Context, Inspectable};
use bevy::prelude::*;
use bevy_rapier3d::{
    na::Isometry3,
    physics::RigidBodyHandleComponent,
    rapier::{
        dynamics::{BodyStatus, MassProperties, RigidBody, RigidBodySet},
        math::Translation,
    },
};

impl_for_simple_enum!(BodyStatus: Dynamic, Static, Kinematic);

impl Inspectable for MassProperties {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _options: Self::Attributes,
        context: &Context,
    ) -> bool {
        let mut changed = false;

        ui.label("Mass");
        let mut mass = 1. / self.inv_mass;
        changed |= mass.ui(ui, NumberAttributes::min(0.001), context);
        self.inv_mass = 1. / mass;
        ui.end_row();

        ui.label("Center of mass");
        let mut com: Vec3 = self.local_com.into();
        changed |= com.ui(ui, Default::default(), context);
        self.local_com = com.into();
        ui.end_row();

        changed
    }
}

impl Inspectable for RigidBody {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _options: Self::Attributes,
        context: &Context,
    ) -> bool {
        // PERF: some updates here can be avoided
        let mut changed = false;
        ui.vertical_centered(|ui| {
            Grid::new(context.id()).show(ui, |ui| {
                ui.label("Body Status");
                let mut body_status = self.body_status();
                changed |= body_status.ui(ui, Default::default(), context);
                self.set_body_status(body_status);
                ui.end_row();

                let mut mass_properties = *self.mass_properties();
                changed |= mass_properties.ui(ui, Default::default(), context);
                self.set_mass_properties(mass_properties, false);

                let position = self.position();

                ui.label("Translation");
                let mut translation: Vec3 = position.translation.vector.into();
                changed |= translation.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("Rotation");
                let mut rotation = position.rotation.to_glam_quat();
                changed |= rotation.ui(ui, Default::default(), context);
                ui.end_row();

                if changed {
                    self.set_position(
                        Isometry3::from_parts(
                            Translation::new(translation.x, translation.y, translation.z),
                            rotation.to_na_unit_quat(),
                        ),
                        false,
                    );
                }

                ui.label("Linear velocity");
                let mut linvel = (*self.linvel()).into();
                trunc_epsilon_vec3(&mut linvel);
                changed |= linvel.ui(ui, Default::default(), context);
                self.set_linvel(linvel.into(), false);
                ui.end_row();

                ui.label("Angular velocity");
                let mut angvel: Vec3 = (*self.angvel()).into();
                trunc_epsilon_vec3(&mut angvel);
                changed |= angvel.ui(ui, Default::default(), context);
                self.set_angvel(angvel.into(), false);
                ui.end_row();

                self.wake_up(false);
            });
        });
        changed
    }
}

fn trunc_epsilon_vec3(val: &mut bevy::math::Vec3) {
    super::trunc_epsilon_f32(&mut val.x);
    super::trunc_epsilon_f32(&mut val.y);
    super::trunc_epsilon_f32(&mut val.z);
}

impl Inspectable for RigidBodyHandleComponent {
    type Attributes = <RigidBody as Inspectable>::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) -> bool {
        let world = expect_world!(ui, context, "RigidBodyHandleComponent");
        let mut bodies = world.get_resource_mut::<RigidBodySet>().unwrap();

        let body = match bodies.get_mut(self.handle()) {
            Some(body) => body,
            None => {
                utils::error_label(ui, "This handle does not exist on RigidBodySet");
                return false;
            }
        };

        body.ui(ui, options, context)
    }
}
