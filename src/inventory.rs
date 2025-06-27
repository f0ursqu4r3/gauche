use crate::item::Item;

/// The fixed number of slots in any inventory.
pub const MAX_SLOTS: usize = 10;

/// Represents a single slot in the inventory, linking a slot index
/// to a unique Item stack with its own state (count, cooldown, etc.).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvEntry {
    pub index: usize,
    pub item: Item,
}

/// Manages a collection of items, handling the logic for adding, stacking,
/// swapping, and removing them within a fixed number of slots.
#[derive(Debug, Clone, Default)]
pub struct Inventory {
    pub entries: Vec<InvEntry>,
    pub selected_index: usize,
}

impl Inventory {
    /// Creates a new, empty inventory with the selected index at 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Unified insert function for adding an item to the inventory.
    ///
    /// # Logic Priority:
    /// 1. **Stack First:** Scans the entire inventory to merge with any compatible, non-full stacks.
    /// 2. **Fill Empty Slot (Overflow):** If the item remains, finds the first available empty slot *prioritizing the selected slot if empty* and places it there.
    /// 3. **Swap with Selected (Last Resort):** Only if the inventory is completely full, it will swap the incoming
    ///    item with the one in the `selected_index` slot.
    ///
    /// # Returns
    /// `None` if the item was fully added.
    /// `Some(Item)` containing the swapped-out item or the un-addable remainder.
    pub fn insert(&mut self, mut item_to_add: Item) -> Option<Item> {
        // --- 1. Prioritize Stacking Globally ---
        if item_to_add.is_stackable() {
            for entry in self.entries.iter_mut() {
                if entry.item.type_ == item_to_add.type_ && entry.item.count < entry.item.max_count
                {
                    let space_available = entry.item.max_count - entry.item.count;
                    let amount_to_transfer = space_available.min(item_to_add.count);
                    entry.item.count += amount_to_transfer;
                    item_to_add.count -= amount_to_transfer;

                    // If we've stacked the entire incoming item, we're done.
                    if item_to_add.count == 0 {
                        return None;
                    }
                }
            }
        }

        // --- 2. If item remains, find an empty slot, prioritizing the selected one ---
        if !self.is_full() {
            // 2a. Prioritize the selected slot if it's empty.
            let is_selected_slot_empty =
                !self.entries.iter().any(|e| e.index == self.selected_index);
            if is_selected_slot_empty {
                self.entries.push(InvEntry {
                    index: self.selected_index, // Use the selected index
                    item: item_to_add,
                });
                self.entries.sort_by_key(|e| e.index); // Keep sorted
                return None; // Item placed in the selected empty slot.
            }

            // 2b. If the selected slot was taken, find any other empty slot.
            if let Some(slot_index) =
                (0..MAX_SLOTS).find(|i| !self.entries.iter().any(|e| e.index == *i))
            {
                self.entries.push(InvEntry {
                    index: slot_index,
                    item: item_to_add,
                });
                self.entries.sort_by_key(|e| e.index); // Keep sorted
                return None; // Item placed in another empty slot.
            }
        }

        // --- 3. Last Resort: Inventory is full, swap with the selected slot ---
        if let Some(pos_in_vec) = self
            .entries
            .iter()
            .position(|e| e.index == self.selected_index)
        {
            // An item exists in the selected slot, so we can swap with it.
            let old_item = self.entries[pos_in_vec].item;
            self.entries[pos_in_vec].item = item_to_add;
            return Some(old_item); // Return the swapped-out item.
        }

        // Failsafe: If inventory is full but the selected slot is somehow empty (which can happen
        // if an item was just used up), this would fail. We can't add the item, so we return it.
        Some(item_to_add)
    }

    /// Check if full.
    pub fn is_full(&self) -> bool {
        self.entries.len() >= MAX_SLOTS
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes a specific number of items from a slot.
    /// If the count of an entry reaches zero, the entry is completely removed.
    pub fn remove_count_from_slot(&mut self, index: usize, count_to_remove: u32) {
        if let Some(entry) = self.get_mut(index) {
            entry.item.count = entry.item.count.saturating_sub(count_to_remove);
            if entry.item.count == 0 {
                self.entries.retain(|e| e.index != index);
            }
        }
    }

    /// Gets an immutable reference to an inventory entry at a specific index.
    pub fn get(&self, index: usize) -> Option<&InvEntry> {
        self.entries.iter().find(|e| e.index == index)
    }

    /// Return true if there is an entry in the selected slot.
    pub fn has_selected_entry(&self) -> bool {
        self.get(self.selected_index).is_some()
    }

    /// Gets a mutable reference to an inventory entry at a specific index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut InvEntry> {
        self.entries.iter_mut().find(|e| e.index == index)
    }

    /// Returns an iterator over the inventory entries.
    pub fn iter(&self) -> std::slice::Iter<'_, InvEntry> {
        self.entries.iter()
    }

    /// Returns a mutable iterator over the items themselves, useful for updating cooldowns.
    pub fn iter_mut_items(&mut self) -> impl Iterator<Item = &mut Item> {
        self.entries.iter_mut().map(|entry| &mut entry.item)
    }

    /// Gets the entry in the currently selected slot.
    pub fn selected_entry(&self) -> Option<&InvEntry> {
        self.get(self.selected_index)
    }

    /// Gets a mutable reference to the entry in the currently selected slot.
    pub fn selected_entry_mut(&mut self) -> Option<&mut InvEntry> {
        self.get_mut(self.selected_index)
    }

    /// Remove selected entry from the inventory.
    pub fn remove_selected_entry(&mut self) -> Option<Item> {
        if let Some(pos) = self
            .entries
            .iter()
            .position(|e| e.index == self.selected_index)
        {
            let removed_entry = self.entries.remove(pos);
            return Some(removed_entry.item);
        }
        None
    }

    /// Sets the selected index, clamping it to the valid range [0, MAX_SLOTS - 1].
    pub fn set_selected_index(&mut self, index: usize) {
        if index < MAX_SLOTS {
            self.selected_index = index;
        }
    }

    /// Moves the selected index to the next slot, wrapping around from 9 to 0.
    pub fn increment_selected_index(&mut self) {
        self.selected_index = (self.selected_index + 1) % MAX_SLOTS;
    }

    /// Moves the selected index to the previous slot, wrapping around from 0 to 9.
    pub fn decrement_selected_index(&mut self) {
        self.selected_index = (self.selected_index + MAX_SLOTS - 1) % MAX_SLOTS;
    }
}
