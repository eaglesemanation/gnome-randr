use std::sync::Arc;

use crate::{
    dbus_api,
    mode_db::{ModeDb, RoundedMode},
};

pub struct Output {
    id: u32,
    possible_modes: Arc<[RoundedMode]>,
}

impl Output {
    pub fn new(dbus_output: &dbus_api::Output, mode_db: &ModeDb) -> Self {
        let possible_modes = mode_db.get_modes_by_ids(&dbus_output.mode_ids);
        Output {
            id: dbus_output.id,
            possible_modes,
        }
    }
}
