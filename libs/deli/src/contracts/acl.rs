use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, B256, U256};

use stylus_sdk::{
    prelude::*,
    storage::{StorageAddress, StorageGuard, StorageMap, StorageU256, StorageVec},
};

#[storage]
pub struct Role {
    assignees: StorageVec<StorageAddress>,
    positions: StorageMap<Address, StorageU256>,
}

impl Role {
    fn assign(&mut self, address: Address) -> Result<(), Vec<u8>> {
        let mut pos_setter = self.positions.setter(address);
        if pos_setter.get().is_zero() {
            self.assignees.push(address);
            let last_pos = self.assignees.len();
            pos_setter.set(U256::from(last_pos));
        } else {
            Err(b"Role already set")?;
        }
        Ok(())
    }

    fn unassign(&mut self, address: Address) -> Result<(), Vec<u8>> {
        let mut pos_setter = self.positions.setter(address);
        let pos = pos_setter.get();
        if pos.is_zero() {
            Err(b"Role not assigned")?;
        }
        pos_setter.erase();
        let last_index = U256::from(self.assignees.len());
        if pos < last_index {
            let last = self.assignees.get(last_index - U256::ONE).unwrap();
            self.assignees.setter(pos - U256::ONE).unwrap().set(last);
            self.positions.setter(last).set(pos);
        }
        self.assignees.erase_last();
        Ok(())
    }

    fn erase_next(&mut self, max_len: usize) -> bool {
        let assignees = self.get_assignees(0, max_len);
        for address in assignees {
            self.unassign(address).unwrap();
        }
        self.assignees.is_empty()
    }

    pub fn contains(&self, address: Address) -> bool {
        !self.positions.get(address).is_zero()
    }

    fn get_assignee_count(&self) -> usize {
        self.assignees.len()
    }

    fn get_assignees(&self, start_from: usize, max_len: usize) -> Vec<Address> {
        let mut result = Vec::with_capacity(max_len);
        let last_index = self.assignees.len();
        if start_from < last_index {
            for index in start_from..last_index {
                let assignee = self.assignees.get(index).unwrap();
                result.push(assignee);
            }
        }
        result
    }
}

/// Lightweight Access Control List (ACL)
///
/// This is constructed from two mappings:
/// * `role => [assignee]` - a set of addresses assigned to each role
/// * `assignee => [role]` - a set of roles assigned to each address
///
/// We can assign to a role multiple assignees, or to an assignee multiple roles.
/// Either way we always update both mappings to keep integrity.
///
#[storage]
pub struct AccessControlList {
    roles: StorageMap<B256, Role>,
}

impl AccessControlList {
    const MAX_LEN: usize = 256;

    pub fn get_role(&self, role: B256) -> StorageGuard<'_, Role> {
        self.roles.get(role)
    }

    pub fn set_role(&mut self, attendee: Address, role: B256) -> Result<(), Vec<u8>> {
        self.roles.setter(role).assign(attendee)?;
        Ok(())
    }

    pub fn only_role(&self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        if !self.has_role(role, attendee) {
            Err(b"Unauthorsed access")?;
        }
        Ok(())
    }

    pub fn has_role(&self, role: B256, attendee: Address) -> bool {
        self.roles.get(role).contains(attendee)
    }

    pub fn unset_role(&mut self, attendee: Address, role: B256) -> Result<(), Vec<u8>> {
        self.roles.setter(role).unassign(attendee)?;
        Ok(())
    }

    pub fn delete_role(&mut self, role: B256) -> bool {
        self.roles.setter(role).erase_next(Self::MAX_LEN)
    }

    pub fn get_role_assignee_count(&self, role: B256) -> usize {
        self.roles.get(role).get_assignee_count()
    }

    pub fn get_role_assignees(
        &self,
        role: B256,
        start_from: usize,
        max_len: usize,
    ) -> Vec<Address> {
        self.roles
            .get(role)
            .get_assignees(start_from, max_len.min(Self::MAX_LEN))
    }
}
