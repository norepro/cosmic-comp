// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    state::State,
    utils::{
        geometry::{PointExt, PointGlobalExt},
        prelude::SeatExt,
    },
};
use smithay::{
    delegate_pointer_constraints,
    input::pointer::PointerHandle,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::{
        pointer_constraints::{PointerConstraintsHandler, with_pointer_constraint},
        seat::WaylandFocus,
    },
};

impl PointerConstraintsHandler for State {
    fn new_constraint(&mut self, _surface: &WlSurface, pointer: &PointerHandle<Self>) {
        self.maybe_activate_pointer_constraint(pointer);
    }

    fn cursor_position_hint(
        &mut self,
        _surface: &WlSurface,
        _pointer: &PointerHandle<Self>,
        _location: Point<f64, Logical>,
    ) {
        // TODO
    }
}

impl State {
    /// Attempt to activate any pointer constraint on the surface under the
    /// pointer at its current location. Called both when a new constraint is
    /// created and during pointer motion so that constraints requested before
    /// the pointer entered the surface are activated lazily.
    ///
    /// # Panics
    /// Must not be called while holding a write lock on `self.common.shell`.
    pub fn maybe_activate_pointer_constraint(&self, pointer: &PointerHandle<Self>) {
        let location = pointer.current_location();

        let shell = self.common.shell.read();
        let seat = shell.seats.last_active();
        let output = seat.active_output();

        let Some((focus_target, surface_loc)) =
            State::surface_under(location.as_global(), &output, &shell)
        else {
            return;
        };
        let Some(surface) = focus_target.wl_surface() else {
            return;
        };

        with_pointer_constraint(&surface, pointer, |constraint| {
            let Some(constraint) = constraint else { return };
            if constraint.is_active() {
                return;
            }

            if let Some(region) = constraint.region() {
                let point = (location - surface_loc.as_logical()).to_i32_round();
                if !region.contains(point) {
                    return;
                }
            }

            constraint.activate();
        });
    }
}

delegate_pointer_constraints!(State);
