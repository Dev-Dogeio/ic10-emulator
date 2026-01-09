//! WASM bindings and JavaScript exports (enabled with the `wasm` feature)

use wasm_bindgen::prelude::*;

use crate::atmospherics::{GasMixture, GasType, MatterState};
use crate::devices::LogicSlotType;
use crate::devices::LogicType;
use crate::devices::device_factory::create_device;
use crate::devices::{Device, SimulationDeviceSettings};
use crate::devices::{DeviceAtmosphericNetworkType, device_factory};
use crate::items::item::Item;
use crate::items::{self, ItemIntegratedCircuit10};
use crate::networks::BatchMode;
use crate::types::{OptShared, Shared, shared};
use crate::{AtmosphericNetwork, CableNetwork, SimulationManager};
use js_sys::Reflect;
use js_sys::{Array, Object};
use std::rc::Rc;

#[wasm_bindgen]
/// WASM wrapper around `CableNetwork`
pub struct WasmCableNetwork {
    inner: Shared<CableNetwork>,
}

#[wasm_bindgen]
impl WasmCableNetwork {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCableNetwork {
        WasmCableNetwork {
            inner: CableNetwork::new(),
        }
    }

    /// Add a `WasmDevice` instance to this cable network.
    pub fn add_device(&self, device: &WasmDevice) -> Result<(), JsValue> {
        self.inner
            .borrow_mut()
            .add_device(device.inner.clone(), self.inner.clone());
        Ok(())
    }

    /// Remove a device from this cable network by reference ID. Returns true if the device was found and removed.
    pub fn remove_device(&self, ref_id: i32) -> bool {
        self.inner.borrow_mut().remove_device(ref_id).is_some()
    }

    /// Get a list of all device reference IDs in this cable network.
    pub fn device_ids(&self) -> Vec<i32> {
        self.inner.borrow().all_device_ids()
    }

    /// Get a device wrapper by reference ID
    pub fn get_device(&self, ref_id: i32) -> Result<WasmDevice, JsValue> {
        let opt = self.inner.borrow().get_device_shared(ref_id);
        opt.map(|d| WasmDevice { inner: d })
            .ok_or_else(|| JsValue::from_str("Device not found"))
    }

    /// Return all devices in this cable network as `WasmDevice` wrappers
    pub fn devices(&self) -> Vec<WasmDevice> {
        self.inner
            .borrow()
            .all_devices()
            .into_iter()
            .map(|d| WasmDevice { inner: d })
            .collect()
    }

    /// Read a logic value from a device by reference ID. `logic_type_value` uses the numeric mapping from `devices::LogicType`.
    pub fn read_device(&self, ref_id: i32, logic_type: LogicType) -> Result<f64, JsValue> {
        let net = self.inner.borrow();
        let device = net
            .get_device(ref_id)
            .ok_or_else(|| JsValue::from_str("Device not found"))?;
        device
            .read(logic_type)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Write a logic value to a device by reference ID. `logic_type_value` uses the numeric mapping from `devices::LogicType`.
    pub fn write_device(
        &self,
        ref_id: i32,
        logic_type: LogicType,
        value: f64,
    ) -> Result<(), JsValue> {
        let net = self.inner.borrow();
        let device = net
            .get_device(ref_id)
            .ok_or_else(|| JsValue::from_str("Device not found"))?;
        device
            .write(logic_type, value)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Set the name of a device by reference ID.
    pub fn set_device_name(&self, ref_id: i32, name: &str) -> Result<(), JsValue> {
        let net = self.inner.borrow();
        let mut dev = net
            .get_device_mut(ref_id)
            .ok_or_else(|| JsValue::from_str("Device not found"))?;
        dev.rename(name);
        Ok(())
    }

    /// Batch read logic values from all devices matching the given prefab hash.
    pub fn batch_read_by_prefab(
        &self,
        prefab_hash: i32,
        logic_type: LogicType,
        batch_mode: BatchMode,
    ) -> Result<f64, JsValue> {
        self.inner
            .borrow()
            .batch_read_by_prefab(prefab_hash, logic_type, batch_mode)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Batch write logic values to all devices matching the given prefab hash.
    pub fn batch_write_by_prefab(
        &self,
        prefab_hash: i32,
        logic_type: LogicType,
        value: f64,
    ) -> Result<usize, JsValue> {
        self.inner
            .borrow()
            .batch_write_by_prefab(prefab_hash, logic_type, value)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Return a string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        format!("{}", self.inner.borrow())
    }

    /// Update a device's name index in the network
    pub fn update_device_name(&self, ref_id: i32, old_name_hash: i32, new_name_hash: i32) {
        self.inner
            .borrow_mut()
            .update_device_name(ref_id, old_name_hash, new_name_hash);
    }

    /// Check whether a device exists on this network
    pub fn device_exists(&self, ref_id: i32) -> bool {
        self.inner.borrow().device_exists(ref_id)
    }

    /// Get all device ids matching a prefab hash
    pub fn get_devices_by_prefab(&self, prefab_hash: i32) -> Vec<i32> {
        self.inner.borrow().get_devices_by_prefab(prefab_hash)
    }

    /// Get all device ids matching a name hash
    pub fn get_devices_by_name(&self, name_hash: i32) -> Vec<i32> {
        self.inner.borrow().get_devices_by_name(name_hash)
    }

    /// Count devices by prefab
    pub fn count_devices_by_prefab(&self, prefab_hash: i32) -> usize {
        self.inner.borrow().count_devices_by_prefab(prefab_hash)
    }

    /// Count devices by name
    pub fn count_devices_by_name(&self, name_hash: i32) -> usize {
        self.inner.borrow().count_devices_by_name(name_hash)
    }

    /// Total devices on this network
    pub fn device_count(&self) -> usize {
        self.inner.borrow().device_count()
    }

    /// Clear the network
    pub fn clear(&self) {
        self.inner.borrow_mut().clear();
    }

    /// Update all devices on the network for the given tick
    pub fn update(&self, tick: u64) {
        self.inner.borrow().update(tick);
    }

    /// Batch read logic values from devices matching prefab and name
    pub fn batch_read_by_name(
        &self,
        prefab_hash: i32,
        name_hash: i32,
        logic_type: LogicType,
        batch_mode: BatchMode,
    ) -> Result<f64, JsValue> {
        self.inner
            .borrow()
            .batch_read_by_name(prefab_hash, name_hash, logic_type, batch_mode)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Batch write logic values to devices matching prefab and name
    pub fn batch_write_by_name(
        &self,
        prefab_hash: i32,
        name_hash: i32,
        logic_type: LogicType,
        value: f64,
    ) -> Result<usize, JsValue> {
        self.inner
            .borrow()
            .batch_write_by_name(prefab_hash, name_hash, logic_type, value)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }
}

impl Default for WasmCableNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WasmCableNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.borrow())
    }
}

#[wasm_bindgen]
/// WASM wrapper around `GasMixture` for direct manipulation
pub struct WasmGasMixture {
    inner: GasMixture,
}

#[wasm_bindgen]
impl WasmGasMixture {
    #[wasm_bindgen(constructor)]
    pub fn new(volume: f64) -> WasmGasMixture {
        WasmGasMixture {
            inner: GasMixture::new(volume),
        }
    }

    pub fn volume(&self) -> f64 {
        self.inner.volume()
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.inner.set_volume(volume);
    }

    pub fn pressure(&self) -> f64 {
        self.inner.pressure()
    }

    pub fn temperature(&self) -> f64 {
        self.inner.temperature()
    }

    pub fn total_moles(&self) -> f64 {
        self.inner.total_moles()
    }

    pub fn total_moles_gases(&self) -> f64 {
        self.inner.total_moles_gases()
    }

    pub fn total_moles_liquids(&self) -> f64 {
        self.inner.total_moles_liquids()
    }

    pub fn get_moles(&self, gas: GasType) -> f64 {
        self.inner.get_moles(gas)
    }

    pub fn gas_ratio(&self, gas: GasType) -> f64 {
        self.inner.gas_ratio(gas)
    }

    pub fn partial_pressure(&self, gas: GasType) -> f64 {
        self.inner.partial_pressure(gas)
    }

    pub fn add_gas(&mut self, gas: GasType, moles: f64, temperature: f64) {
        self.inner.add_gas(gas, moles, temperature);
    }

    pub fn remove_gas(&mut self, gas: GasType, moles: f64) -> f64 {
        let removed = self.inner.remove_gas(gas, moles);
        removed.quantity()
    }

    pub fn remove_all_gas(&mut self, gas: GasType) -> f64 {
        let removed = self.inner.remove_all_gas(gas);
        removed.quantity()
    }

    pub fn process_phase_changes(&mut self) -> u32 {
        self.inner.process_phase_changes()
    }

    pub fn transfer_ratio_to(
        &mut self,
        target: &mut WasmGasMixture,
        ratio: f64,
        state_value: u32,
    ) -> f64 {
        let state = MatterState::from_value(state_value).unwrap_or(MatterState::None);
        self.inner
            .transfer_ratio_to(&mut target.inner, ratio, state)
    }

    pub fn equalize_with(&mut self, other: &mut WasmGasMixture) {
        self.inner.equalize_with(&mut other.inner);
    }

    pub fn remove_moles(&mut self, moles: f64, state_value: u32) -> WasmGasMixture {
        let state = MatterState::from_value(state_value).unwrap_or(MatterState::None);
        let removed = self.inner.remove_moles(moles, state);
        WasmGasMixture { inner: removed }
    }

    pub fn merge(&mut self, other: &WasmGasMixture) {
        self.inner.merge(&other.inner);
    }

    pub fn merge_by_state(&mut self, other: &WasmGasMixture, state_value: u32) {
        let state = MatterState::from_value(state_value).unwrap_or(MatterState::None);
        self.inner.merge_by_state(&other.inner, state);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn clear_gases(&mut self) {
        self.inner.clear_gases();
    }

    pub fn clear_liquids(&mut self) {
        self.inner.clear_liquids();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn scale(&mut self, ratio: f64, state_value: u32) {
        let state = MatterState::from_value(state_value).unwrap_or(MatterState::None);
        self.inner.scale(ratio, state);
    }

    /// Return a string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        format!("{}", self.inner)
    }

    pub fn total_moles_by_state(&self, state_value: u32) -> f64 {
        let state = MatterState::from_value(state_value).unwrap_or(MatterState::None);
        self.inner.total_moles_by_state(state)
    }

    pub fn total_volume_liquids(&self) -> f64 {
        self.inner.total_volume_liquids()
    }

    pub fn liquid_volume_ratio(&self) -> f64 {
        self.inner.liquid_volume_ratio()
    }

    pub fn gas_volume(&self) -> f64 {
        self.inner.gas_volume()
    }

    pub fn total_energy_gases(&self) -> f64 {
        self.inner.total_energy_gases()
    }

    pub fn total_energy_liquids(&self) -> f64 {
        self.inner.total_energy_liquids()
    }

    pub fn total_energy(&self) -> f64 {
        self.inner.total_energy()
    }

    pub fn total_heat_capacity_gases(&self) -> f64 {
        self.inner.total_heat_capacity_gases()
    }

    pub fn total_heat_capacity_liquids(&self) -> f64 {
        self.inner.total_heat_capacity_liquids()
    }

    pub fn total_heat_capacity(&self) -> f64 {
        self.inner.total_heat_capacity()
    }

    pub fn pressure_gases(&self) -> f64 {
        self.inner.pressure_gases()
    }

    pub fn add_energy(&mut self, joules: f64) {
        self.inner.add_energy(joules);
    }

    pub fn remove_energy(&mut self, joules: f64) -> f64 {
        self.inner.remove_energy(joules)
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.inner.set_temperature(temperature);
    }

    pub fn equalize_internal_energy(&mut self) {
        self.inner.equalize_internal_energy();
    }
}

#[wasm_bindgen]
/// WASM wrapper around any `Device`
pub struct WasmDevice {
    inner: Shared<dyn Device>,
}

#[wasm_bindgen]
impl WasmDevice {
    /// Get device reference id
    pub fn id(&self) -> i32 {
        self.inner.borrow().get_id()
    }

    /// Get prefab hash for this device
    pub fn prefab_hash(&self) -> i32 {
        self.inner.borrow().get_prefab_hash()
    }

    /// Get device name
    pub fn name(&self) -> String {
        self.inner.borrow().get_name().to_string()
    }

    /// Set device name
    pub fn rename(&self, name: &str) {
        self.inner.borrow_mut().rename(name);
    }

    /// Return a string representation of the device
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        format!("{:?}", self.inner.borrow())
    }

    /// Check whether this device can be read for a given logic type
    pub fn can_read(&self, logic_type: LogicType) -> bool {
        self.inner.borrow().can_read(logic_type)
    }

    /// Check whether this device can be written for a given logic type
    pub fn can_write(&self, logic_type: LogicType) -> bool {
        self.inner.borrow().can_write(logic_type)
    }

    /// Get the name hash for this device
    pub fn name_hash(&self) -> i32 {
        self.inner.borrow().get_name_hash()
    }

    /// Get the cable network this device is connected to (if any)
    pub fn get_network(&self) -> Option<WasmCableNetwork> {
        self.inner
            .borrow()
            .get_network()
            .map(|n| WasmCableNetwork { inner: n })
    }

    /// Memory access helpers (delegates to device's memory methods if available)
    pub fn get_memory(&self, index: usize) -> Result<f64, JsValue> {
        self.inner
            .borrow()
            .get_memory(index)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    pub fn set_memory(&self, index: usize, value: f64) -> Result<(), JsValue> {
        self.inner
            .borrow()
            .set_memory(index, value)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Clear device memory/stack if supported
    pub fn clear_memory(&self) -> Result<(), JsValue> {
        self.inner
            .borrow()
            .clear()
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    pub fn read(&self, logic_type: LogicType) -> Result<f64, JsValue> {
        self.inner
            .borrow()
            .read(logic_type)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    pub fn write(&self, logic_type: LogicType, value: f64) -> Result<(), JsValue> {
        self.inner
            .borrow()
            .write(logic_type, value)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Return supported logic types for this device as numeric values
    pub fn supported_types(&self) -> Vec<f64> {
        self.inner
            .borrow()
            .supported_types()
            .into_iter()
            .map(|lt| (lt as i32) as f64)
            .collect()
    }

    /// Return supported slot logic types for this device as numeric values
    pub fn supported_slot_types(&self) -> Vec<f64> {
        self.inner
            .borrow()
            .supported_slot_types()
            .into_iter()
            .map(|lt| (lt as i32) as f64)
            .collect()
    }

    /// Read a batch of logic types and return an array of values (null for failures)
    pub fn read_batch(&self, logic_types: Vec<LogicType>) -> Result<Array, JsValue> {
        let arr = Array::new();
        let device = self.inner.borrow();
        for lt in logic_types {
            match device.read(lt) {
                Ok(val) => arr.push(&JsValue::from_f64(val)),
                Err(_) => arr.push(&JsValue::NULL),
            };
        }
        Ok(arr)
    }

    /// Add this device to a cable network
    pub fn add_to_network(&self, network: &WasmCableNetwork) -> Result<(), JsValue> {
        network
            .inner
            .borrow_mut()
            .add_device(self.inner.clone(), network.inner.clone());
        Ok(())
    }

    /// Remove this device from its current network (if any)
    pub fn remove_from_network(&self) -> Result<bool, JsValue> {
        if let Some(net) = self.inner.borrow().get_network() {
            let id = self.inner.borrow().get_id();
            Ok(net.borrow_mut().remove_device(id).is_some())
        } else {
            Ok(false)
        }
    }

    /// Set an atmospheric network connection on this device.
    /// `connection` maps to `DeviceAtmosphericNetworkType` (0=Input,1=Input2,2=Output,3=Output2,4=Internal).
    pub fn set_atmospheric_network(
        &self,
        connection: f64,
        network: &WasmAtmosphericNetwork,
    ) -> Result<(), JsValue> {
        let conn = DeviceAtmosphericNetworkType::from_value(connection as i32)
            .ok_or_else(|| JsValue::from_str("Invalid DeviceAtmosphericNetworkType value"))?;

        let mut dev = self.inner.borrow_mut();
        if let Some(atm) = dev.as_atmospheric_device_mut() {
            atm.set_atmospheric_network(conn, Some(network.inner.clone()))
                .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
            Ok(())
        } else {
            Err(JsValue::from_str(
                "Device does not support atmospheric network connections",
            ))
        }
    }

    /// Run the device's run() method (execute chips if supported)
    pub fn run(&self) -> Result<(), JsValue> {
        self.inner
            .borrow()
            .run()
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Update the device for a given tick (calls `update`)
    pub fn update(&self, tick: u64) -> Result<(), JsValue> {
        self.inner
            .borrow()
            .update(tick)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Read a slot logic value from this device.
    pub fn read_slot(&self, index: usize, slot_logic_type: LogicSlotType) -> Result<f64, JsValue> {
        self.inner
            .borrow()
            .read_slot(index, slot_logic_type)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Write a slot logic value to this device.
    pub fn write_slot(
        &self,
        index: usize,
        slot_logic_type: LogicSlotType,
        value: f64,
    ) -> Result<(), JsValue> {
        self.inner
            .borrow()
            .write_slot(index, slot_logic_type, value)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Insert an item into a slot on this device. The item is consumed on success.
    /// Accepts a `WasmItem` which owns the shared `Item` instance.
    ///
    /// Returns Ok(None) on successful insertion, or Ok(Some(WasmItem)) if the item
    /// could not be inserted (leftover returned to the caller). Errors indicate
    /// invalid/consumed inputs.
    pub fn insert_item_into_slot(
        &self,
        index: usize,
        mut item: WasmItem,
    ) -> Result<Option<WasmItem>, JsValue> {
        // Take the shared item out of the wrapper (ownership move)
        let shared_item = item
            .take()
            .ok_or_else(|| JsValue::from_str("Invalid or already-consumed item"))?;

        // Limit the borrow of the device while inserting
        let result = {
            let mut dev = self.inner.borrow_mut();
            if let Some(slot_host) = dev.as_slot_host_device_mut() {
                slot_host.try_insert_item(index, shared_item)
            } else {
                // Device doesn't support slots; return the shared item as Err so we can hand it back to JS
                Err(shared_item)
            }
        };

        match result {
            Ok(()) => Ok(None),
            Err(leftover) => Ok(Some(WasmItem {
                inner: Some(leftover),
            })),
        }
    }

    /// Remove an item from a slot and return its wasm item id if removed
    pub fn remove_item_from_slot(&self, index: usize) -> Result<Option<WasmItem>, JsValue> {
        // Limit the borrow of the device
        let opt_item = {
            let mut dev = self.inner.borrow_mut();
            if let Some(slot_host) = dev.as_slot_host_device_mut() {
                slot_host.remove_item(index)
            } else {
                return Err(JsValue::from_str("Device does not support slots"));
            }
        };

        if let Some(item) = opt_item {
            Ok(Some(WasmItem { inner: Some(item) }))
        } else {
            Ok(None)
        }
    }

    /// Install an IC chip into this device (if supported).
    /// Accepts a `WasmICChip` instance which wraps a `Shared<ItemIntegratedCircuit10>`.
    pub fn set_chip(&self, chip: &WasmICChip) -> Result<(), JsValue> {
        let dev = self.inner.borrow();
        if let Some(host) = dev.as_ic_host_device() {
            host.set_chip(chip.inner.clone());
            Ok(())
        } else {
            Err(JsValue::from_str("Device does not support IC installation"))
        }
    }
}

#[wasm_bindgen]
pub struct WasmItem {
    inner: OptShared<dyn Item>,
}

#[wasm_bindgen]
impl WasmItem {
    /// Get the item's id (0 if already consumed)
    pub fn id(&self) -> i32 {
        self.inner
            .as_ref()
            .map(|b| b.borrow().get_id())
            .unwrap_or(0)
    }

    /// Get the prefab hash for this item (0 if consumed)
    pub fn prefab_hash(&self) -> i32 {
        self.inner
            .as_ref()
            .map(|b| b.borrow().get_prefab_hash())
            .unwrap_or(0)
    }

    /// Get the quantity of this item (0 if consumed)
    pub fn quantity(&self) -> u32 {
        self.inner
            .as_ref()
            .map(|b| b.borrow().quantity())
            .unwrap_or(0)
    }

    /// Set the quantity (returns true if successful)
    pub fn set_quantity(&mut self, quantity: u32) -> bool {
        if let Some(s) = &self.inner {
            s.borrow_mut().set_quantity(quantity)
        } else {
            false
        }
    }

    /// Maximum stack quantity for this item
    pub fn max_quantity(&self) -> u32 {
        self.inner
            .as_ref()
            .map(|b| b.borrow().max_quantity())
            .unwrap_or(0)
    }

    /// Merge another item into this one; consumes the other item and returns whether any merged
    pub fn merge(&mut self, other: WasmItem) -> Result<bool, JsValue> {
        let a = self
            .inner
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Invalid item"))?;
        let other_inner = other
            .into_inner()
            .ok_or_else(|| JsValue::from_str("Invalid item"))?;
        let mut other_borrow = other_inner.borrow_mut();
        let mut a_borrow = a.borrow_mut();
        Ok(a_borrow.merge(&mut *other_borrow))
    }

    /// Get the item type as a string
    pub fn item_type(&self) -> String {
        self.inner
            .as_ref()
            .map(|b| b.borrow().item_type().as_str().to_string())
            .unwrap_or_default()
    }
}

impl WasmItem {
    /// Consume the wrapper and return the shared item (used by insert_item_into_slot)
    pub fn into_inner(mut self) -> OptShared<dyn Item> {
        self.inner.take()
    }

    /// Take ownership of the inner shared item without consuming self (internal use)
    fn take(&mut self) -> OptShared<dyn Item> {
        self.inner.take()
    }
}

#[wasm_bindgen]
pub struct WasmICChip {
    inner: Shared<ItemIntegratedCircuit10>,
}

#[wasm_bindgen]
impl WasmICChip {
    /// Create a new IC chip
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmICChip {
        WasmICChip {
            inner: shared(ItemIntegratedCircuit10::new(None)),
        }
    }

    pub fn id(&self) -> i32 {
        self.inner.borrow().get_id()
    }

    pub fn load_program(&self, source: &str) -> Result<(), JsValue> {
        self.inner
            .borrow_mut()
            .load_program(source)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    pub fn step(&self) -> Result<bool, JsValue> {
        self.inner
            .borrow()
            .step()
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    pub fn run(&self, max_steps: usize) -> Result<usize, JsValue> {
        self.inner
            .borrow()
            .run(max_steps)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    pub fn get_pc(&self) -> usize {
        self.inner.borrow().get_pc()
    }

    pub fn is_halted(&self) -> bool {
        self.inner.borrow().is_halted()
    }

    pub fn get_memory(&self, index: usize) -> Result<f64, JsValue> {
        self.inner
            .borrow()
            .read_stack(index)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    pub fn set_memory(&self, index: usize, value: f64) -> Result<(), JsValue> {
        self.inner
            .borrow()
            .write_stack(index, value)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    pub fn insert_define(&self, name: &str, value: f64) {
        self.inner.borrow().insert_define(name, value);
    }

    pub fn insert_alias(&self, name: &str, device_ref_id: i32) {
        self.inner
            .borrow()
            .add_device_alias(name.to_string(), device_ref_id);
    }

    pub fn get_host_id(&self) -> Option<i32> {
        self.inner.borrow().get_host_id()
    }
}

impl Default for WasmICChip {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
pub struct WasmAtmosphericNetwork {
    inner: Shared<AtmosphericNetwork>,
}

#[wasm_bindgen]
impl WasmAtmosphericNetwork {
    #[wasm_bindgen(constructor)]
    pub fn new(volume: f64) -> WasmAtmosphericNetwork {
        WasmAtmosphericNetwork {
            inner: AtmosphericNetwork::new(volume),
        }
    }

    pub fn pressure(&self) -> f64 {
        self.inner.borrow().pressure()
    }

    pub fn temperature(&self) -> f64 {
        self.inner.borrow().temperature()
    }

    pub fn total_moles(&self) -> f64 {
        self.inner.borrow().total_moles()
    }

    pub fn add_gas(&self, gas: GasType, moles: f64, temperature: f64) -> Result<(), JsValue> {
        self.inner.borrow_mut().add_gas(gas, moles, temperature);
        Ok(())
    }

    pub fn remove_gas(&self, gas: GasType, moles: f64) -> Result<f64, JsValue> {
        Ok(self.inner.borrow_mut().remove_gas(gas, moles))
    }

    pub fn set_volume(&self, volume: f64) {
        self.inner.borrow_mut().set_volume(volume);
    }

    pub fn clear(&self) {
        self.inner.borrow_mut().clear();
    }

    /// Return a string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        format!("{}", self.inner.borrow())
    }

    pub fn total_volume(&self) -> f64 {
        self.inner.borrow().total_volume()
    }

    pub fn device_count(&self) -> usize {
        self.inner.borrow().device_count()
    }

    pub fn has_device(&self, device_id: usize) -> bool {
        self.inner.borrow().has_device(device_id)
    }

    pub fn device_ids(&self) -> Vec<usize> {
        self.inner.borrow().device_ids()
    }

    pub fn add_mixture(&self, other: &WasmGasMixture) {
        self.inner.borrow_mut().add_mixture(&other.inner);
    }

    pub fn remove_moles(&self, moles: f64, state_value: u32) -> WasmGasMixture {
        let state = MatterState::from_value(state_value).unwrap_or(MatterState::None);
        let removed = self.inner.borrow_mut().remove_moles(moles, state);
        WasmGasMixture { inner: removed }
    }

    pub fn remove_all_gas(&self, gas: GasType) -> f64 {
        let removed = self.inner.borrow_mut().remove_all_gas(gas);
        removed.quantity()
    }

    pub fn process_phase_changes(&self) -> u32 {
        self.inner.borrow_mut().process_phase_changes()
    }

    pub fn gas_ratio(&self, gas: GasType) -> f64 {
        self.inner.borrow().gas_ratio(gas)
    }

    pub fn partial_pressure(&self, gas: GasType) -> f64 {
        self.inner.borrow().partial_pressure(gas)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }

    pub fn merge_network(&self, other: &mut WasmAtmosphericNetwork) -> Vec<usize> {
        self.inner
            .borrow_mut()
            .merge_network(&mut other.inner.borrow_mut())
    }

    pub fn equalize_with(&self, other: &mut WasmAtmosphericNetwork) {
        self.inner
            .borrow_mut()
            .equalize_with(&mut other.inner.borrow_mut());
    }

    pub fn transfer_to(&self, other: &mut WasmAtmosphericNetwork, moles: f64) {
        self.inner
            .borrow_mut()
            .transfer_to(&mut other.inner.borrow_mut(), moles);
    }

    pub fn set_temperature(&self, temperature: f64) {
        self.inner.borrow_mut().set_temperature(temperature);
    }

    pub fn add_energy(&self, joules: f64) {
        self.inner.borrow_mut().add_energy(joules);
    }

    pub fn remove_energy(&self, joules: f64) -> f64 {
        self.inner.borrow_mut().remove_energy(joules)
    }

    // ---- GasMixture parity methods (delegating to the internal mixture) ----
    pub fn volume(&self) -> f64 {
        self.inner.borrow().mixture().volume()
    }

    pub fn total_moles_gases(&self) -> f64 {
        self.inner.borrow().mixture().total_moles_gases()
    }

    pub fn total_moles_liquids(&self) -> f64 {
        self.inner.borrow().mixture().total_moles_liquids()
    }

    pub fn total_moles_by_state(&self, state_value: u32) -> f64 {
        let state = MatterState::from_value(state_value).unwrap_or(MatterState::None);
        self.inner.borrow().mixture().total_moles_by_state(state)
    }

    pub fn total_volume_liquids(&self) -> f64 {
        self.inner.borrow().mixture().total_volume_liquids()
    }

    pub fn liquid_volume_ratio(&self) -> f64 {
        self.inner.borrow().mixture().liquid_volume_ratio()
    }

    pub fn gas_volume(&self) -> f64 {
        self.inner.borrow().mixture().gas_volume()
    }

    pub fn total_energy_gases(&self) -> f64 {
        self.inner.borrow().mixture().total_energy_gases()
    }

    pub fn total_energy_liquids(&self) -> f64 {
        self.inner.borrow().mixture().total_energy_liquids()
    }

    pub fn total_energy(&self) -> f64 {
        self.inner.borrow().mixture().total_energy()
    }

    pub fn total_heat_capacity_gases(&self) -> f64 {
        self.inner.borrow().mixture().total_heat_capacity_gases()
    }

    pub fn total_heat_capacity_liquids(&self) -> f64 {
        self.inner.borrow().mixture().total_heat_capacity_liquids()
    }

    pub fn total_heat_capacity(&self) -> f64 {
        self.inner.borrow().mixture().total_heat_capacity()
    }

    pub fn pressure_gases(&self) -> f64 {
        self.inner.borrow().mixture().pressure_gases()
    }

    pub fn equalize_internal_energy(&self) {
        self.inner
            .borrow_mut()
            .mixture_mut()
            .equalize_internal_energy();
    }

    pub fn scale(&self, ratio: f64, state_value: u32) {
        let state = MatterState::from_value(state_value).unwrap_or(MatterState::None);
        self.inner.borrow_mut().mixture_mut().scale(ratio, state);
    }

    /// Transfer a ratio of moles from this network's mixture to another network's mixture
    pub fn transfer_ratio_to_network(
        &self,
        target: &WasmAtmosphericNetwork,
        ratio: f64,
        state_value: u32,
    ) -> f64 {
        let state = MatterState::from_value(state_value).unwrap_or(MatterState::None);
        if Rc::ptr_eq(&self.inner, &target.inner) {
            return 0.0;
        }

        // Deterministic borrow ordering by pointer address
        let (a_rc, b_rc) =
            if (Rc::as_ptr(&self.inner) as usize) <= (Rc::as_ptr(&target.inner) as usize) {
                (&self.inner, &target.inner)
            } else {
                (&target.inner, &self.inner)
            };

        let mut a = a_rc.borrow_mut();
        let mut b = b_rc.borrow_mut();
        a.mixture_mut()
            .transfer_ratio_to(b.mixture_mut(), ratio, state)
    }
}
#[wasm_bindgen]
pub struct WasmSimulationManager;

#[wasm_bindgen]
impl WasmSimulationManager {
    pub fn reset() {
        SimulationManager::reset_global();
    }

    pub fn update(ticks: u64) -> u32 {
        SimulationManager::update_global(ticks)
    }

    pub fn cable_network_count() -> usize {
        SimulationManager::cable_network_count_global()
    }

    pub fn atmospheric_network_count() -> usize {
        SimulationManager::atmospheric_network_count_global()
    }

    /// Get a string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js() -> String {
        format!("{}", SimulationManager::global().borrow())
    }

    /// Create a device by prefab hash and register it in the global device map.
    /// Returns a `WasmDevice` wrapper for the created instance.
    pub fn create_device(prefab_hash: i32) -> Result<WasmDevice, JsValue> {
        match create_device(prefab_hash, None) {
            Some(d) => Ok(WasmDevice { inner: d }),
            None => Err(JsValue::from_str(
                "Unsupported prefab hash for device creation",
            )),
        }
    }

    /// Create a device with explicit simulation settings
    pub fn create_device_with_settings(
        prefab_hash: i32,
        id: Option<i32>,
        name: Option<String>,
        internal_atmospheric_network: Option<WasmAtmosphericNetwork>,
        ticks_per_day: Option<f64>,
        max_instructions_per_tick: Option<usize>,
    ) -> Result<WasmDevice, JsValue> {
        let settings = SimulationDeviceSettings {
            id,
            name,
            internal_atmospheric_network: internal_atmospheric_network.map(|n| n.inner.clone()),
            ticks_per_day,
            max_instructions_per_tick,
        };

        match create_device(prefab_hash, Some(settings)) {
            Some(d) => Ok(WasmDevice { inner: d }),
            None => Err(JsValue::from_str(
                "Unsupported prefab hash for device creation",
            )),
        }
    }

    /// Create an item by `prefab_hash` and return a `WasmItem` wrapper (not registered).
    /// Supports IC10 and filter prefabs by probing known variants.
    pub fn create_item(prefab_hash: i32) -> Result<WasmItem, JsValue> {
        if let Some(item) = items::create_item(prefab_hash, None) {
            return Ok(WasmItem { inner: Some(item) });
        }

        Err(JsValue::from_str(
            "Unsupported prefab hash for item creation",
        ))
    }
}

/// Return a list of registered device prefab hashes
#[wasm_bindgen]
pub fn get_registered_device_prefabs() -> Vec<i32> {
    device_factory::get_registered_device_prefabs()
}

/// Get comprehensive prefab info for a prefab: returns an object
/// {
///   device_name,
///   prefab_hash,
///   properties: [{ logic, logic_name, readable, writable }],
///   slot_properties: [{ slot_logic, slot_logic_name, readable, slot_ids }],
///   atmospheric_connections: [{ name }]
/// }
#[wasm_bindgen]
pub fn get_prefab_info(prefab_hash: i32) -> Result<Object, JsValue> {
    if let Some((device_name, props)) = device_factory::get_prefab_metadata(prefab_hash) {
        let obj = Object::new();
        Reflect::set(
            &obj,
            &JsValue::from_str("device_name"),
            &JsValue::from_str(device_name),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &JsValue::from_str("prefab_hash"),
            &JsValue::from_f64(prefab_hash as f64),
        )
        .unwrap();

        // properties
        let props_arr = Array::new();
        for (lt, readable, writable) in props.properties {
            let p = Object::new();
            Reflect::set(
                &p,
                &JsValue::from_str("logic"),
                &JsValue::from_f64((lt as i32) as f64),
            )
            .unwrap();
            Reflect::set(
                &p,
                &JsValue::from_str("logic_name"),
                &JsValue::from_str(&format!("{:?}", lt)),
            )
            .unwrap();
            Reflect::set(
                &p,
                &JsValue::from_str("readable"),
                &JsValue::from_bool(readable),
            )
            .unwrap();
            Reflect::set(
                &p,
                &JsValue::from_str("writable"),
                &JsValue::from_bool(writable),
            )
            .unwrap();
            props_arr.push(&JsValue::from(p));
        }
        Reflect::set(&obj, &JsValue::from_str("properties"), &props_arr).unwrap();

        // slot_properties
        let slot_props_arr = Array::new();
        for (lt, readable, slot_ids) in props.slot_properties {
            let sp = Object::new();
            Reflect::set(
                &sp,
                &JsValue::from_str("slot_logic"),
                &JsValue::from_f64((lt as i32) as f64),
            )
            .unwrap();
            Reflect::set(
                &sp,
                &JsValue::from_str("slot_logic_name"),
                &JsValue::from_str(&format!("{:?}", lt)),
            )
            .unwrap();
            Reflect::set(
                &sp,
                &JsValue::from_str("readable"),
                &JsValue::from_bool(readable),
            )
            .unwrap();
            let slot_arr = Array::new();
            for s in slot_ids {
                slot_arr.push(&JsValue::from_f64(s as f64));
            }
            Reflect::set(&sp, &JsValue::from_str("slot_ids"), &slot_arr).unwrap();
            slot_props_arr.push(&JsValue::from(sp));
        }
        Reflect::set(&obj, &JsValue::from_str("slot_properties"), &slot_props_arr).unwrap();

        let atmo_conn_arr = Array::new();
        for c in props.atmospheric_connections {
            let conn = Object::new();
            Reflect::set(
                &conn,
                &JsValue::from_str("name"),
                &JsValue::from_str(&format!("{:?}", c)),
            )
            .unwrap();
            atmo_conn_arr.push(&JsValue::from(conn));
        }
        Reflect::set(
            &obj,
            &JsValue::from_str("atmospheric_connections"),
            &atmo_conn_arr,
        )
        .unwrap();

        Ok(obj)
    } else {
        Err(JsValue::from_str("Unknown prefab hash"))
    }
}

/// Return all registered item prefab hashes
#[wasm_bindgen]
pub fn get_registered_item_prefabs() -> Vec<i32> {
    items::get_registered_item_prefabs()
}

/// Get item prefab metadata: returns an object with { name, prefab_hash, item_type }
#[wasm_bindgen]
pub fn get_item_prefab_info(prefab_hash: i32) -> Result<Object, JsValue> {
    if let Some((name, item_type)) = items::get_prefab_metadata(prefab_hash) {
        let obj = Object::new();
        Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(name)).unwrap();
        Reflect::set(
            &obj,
            &JsValue::from_str("prefab_hash"),
            &JsValue::from_f64(prefab_hash as f64),
        )
        .unwrap();
        Reflect::set(
            &obj,
            &JsValue::from_str("item_type"),
            &JsValue::from_str(item_type.as_str()),
        )
        .unwrap();
        return Ok(obj);
    }

    Err(JsValue::from_str(
        "Unsupported prefab hash for item prefab info",
    ))
}
