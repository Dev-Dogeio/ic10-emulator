//! WASM bindings and JavaScript exports (enabled with the `wasm` feature)

use wasm_bindgen::prelude::*;

use crate::atmospherics::{GasMixture, GasType, MatterState};
use crate::devices::Device;
use crate::devices::DeviceAtmosphericNetworkType;
use crate::devices::LogicSlotType;
use crate::devices::LogicType;
use crate::devices::{
    AirConditioner, DaylightSensor, Filtration, ICHousing, LogicMemory, VolumePump,
};
use crate::items::item::Item;
use crate::items::{Filter, ItemIntegratedCircuit10};
use crate::networks::BatchMode;
use crate::types::{OptShared, Shared};
use crate::{AtmosphericNetwork, CableNetwork, SimulationManager};

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static WASM_DEVICE_REGISTRY: RefCell<HashMap<i32, Shared<dyn Device>>> = RefCell::new(HashMap::new());
    static WASM_ITEM_REGISTRY: RefCell<HashMap<i32, Shared<dyn Item>>> = RefCell::new(HashMap::new());
    /// Separate registry for IC chips to preserve their concrete type
    static WASM_IC_REGISTRY: RefCell<HashMap<i32, Shared<ItemIntegratedCircuit10>>> = RefCell::new(HashMap::new());
}

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
        dev.set_name(name);
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
    pub fn set_name(&self, name: &str) {
        self.inner.borrow_mut().set_name(name);
    }

    /// Return a string representation of the device
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        format!("{:?}", self.inner.borrow())
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
        if let Some(atm) = dev.as_atmospheric_device() {
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
    pub fn insert_item_into_slot(&self, index: usize, mut item: WasmItem) -> Result<(), JsValue> {
        // Take the shared item out of the wrapper (ownership move)
        let shared_item = item
            .take()
            .ok_or_else(|| JsValue::from_str("Invalid or already-consumed item"))?;

        // Limit the borrow of the device so we don't hold it while touching registries
        let result = {
            let mut dev = self.inner.borrow_mut();
            if let Some(slot_host) = dev.as_slot_host_device() {
                slot_host.try_insert_item(index, shared_item)
            } else {
                // Device doesn't support slots; return the shared item as Err so we can put it back in registry
                Err(shared_item)
            }
        };

        match result {
            Ok(()) => Ok(()),
            Err(leftover) => {
                // Put leftover back in the registry so JS can still access it by id
                let id = leftover.borrow().get_id();
                WASM_ITEM_REGISTRY.with(|r| r.borrow_mut().insert(id, leftover));
                Err(JsValue::from_str(
                    "Item insertion failed or device doesn't support slots",
                ))
            }
        }
    }

    /// Remove an item from a slot and return its wasm item id if removed
    pub fn remove_item_from_slot(&self, index: usize) -> Result<Option<i32>, JsValue> {
        // Limit the borrow of the device
        let opt_item = {
            let mut dev = self.inner.borrow_mut();
            if let Some(slot_host) = dev.as_slot_host_device() {
                slot_host.remove_item(index)
            } else {
                return Err(JsValue::from_str("Device does not support slots"));
            }
        };

        if let Some(item) = opt_item {
            let id = item.borrow().get_id();
            WASM_ITEM_REGISTRY.with(|r| r.borrow_mut().insert(id, item));
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    /// Install an IC chip into this device (if supported).
    /// Accepts a `WasmICChip` instance which wraps a `Shared<ItemIntegratedCircuit10>`.
    pub fn set_chip(&self, chip: &WasmICChip) -> Result<(), JsValue> {
        let mut dev = self.inner.borrow_mut();
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
    /// Create a new IC chip and register it
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmICChip {
        let chip = crate::types::shared(ItemIntegratedCircuit10::new());
        let id = chip.borrow().get_id();
        WASM_IC_REGISTRY.with(|r| r.borrow_mut().insert(id, chip.clone()));
        WasmICChip { inner: chip }
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
}

#[wasm_bindgen]
pub struct WasmSimulationManager;

#[wasm_bindgen]
impl WasmSimulationManager {
    pub fn reset() {
        SimulationManager::reset_global();

        // Clear wasm-side device registry as well
        WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().clear());
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
        // Try known device constructors and match prefab hash
        // Check static prefab hashes before creating instances
        if VolumePump::prefab_hash() == prefab_hash {
            let d = VolumePump::new(None);
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if AirConditioner::prefab_hash() == prefab_hash {
            let d = AirConditioner::new(None);
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if Filtration::prefab_hash() == prefab_hash {
            let d = Filtration::new(None);
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if DaylightSensor::prefab_hash() == prefab_hash {
            let d = DaylightSensor::new(None);
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if ICHousing::prefab_hash() == prefab_hash {
            let d = ICHousing::new(None);
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if LogicMemory::prefab_hash() == prefab_hash {
            let d = LogicMemory::new(None);
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        Err(JsValue::from_str(
            "Unsupported prefab hash for device creation",
        ))
    }

    /// Create a device with explicit simulation settings
    pub fn create_device_with_settings(
        prefab_hash: i32,
        ticks_per_day: f64,
        max_instructions_per_tick: usize,
    ) -> Result<WasmDevice, JsValue> {
        let settings = crate::devices::SimulationSettings {
            ticks_per_day,
            max_instructions_per_tick,
        };

        if VolumePump::prefab_hash() == prefab_hash {
            let d = VolumePump::new(Some(settings.clone()));
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if AirConditioner::prefab_hash() == prefab_hash {
            let d = AirConditioner::new(Some(settings.clone()));
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if Filtration::prefab_hash() == prefab_hash {
            let d = Filtration::new(Some(settings.clone()));
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if DaylightSensor::prefab_hash() == prefab_hash {
            let d = DaylightSensor::new(Some(settings.clone()));
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if ICHousing::prefab_hash() == prefab_hash {
            let d = ICHousing::new(Some(settings.clone()));
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        if LogicMemory::prefab_hash() == prefab_hash {
            let d = LogicMemory::new(Some(settings.clone()));
            let id = d.borrow().get_id();
            WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().insert(id, d.clone()));
            return Ok(WasmDevice { inner: d });
        }

        Err(JsValue::from_str(
            "Unsupported prefab hash for device creation",
        ))
    }

    /// Create an item by `prefab_hash` and return a `WasmItem` wrapper (not registered).
    /// Supports IC10 and filter prefabs by probing known variants.
    pub fn create_item(prefab_hash: i32) -> Result<WasmItem, JsValue> {
        // Try IC10
        if crate::items::ItemIntegratedCircuit10::prefab_hash() == prefab_hash {
            let chip: Shared<dyn Item> = crate::types::shared(ItemIntegratedCircuit10::new());
            return Ok(WasmItem { inner: Some(chip) });
        }

        // Try filters by enumerating gas types and sizes
        use crate::items::FilterSize;
        let gas_types = [
            crate::atmospherics::GasType::Oxygen,
            crate::atmospherics::GasType::Nitrogen,
            crate::atmospherics::GasType::CarbonDioxide,
            crate::atmospherics::GasType::Volatiles,
            crate::atmospherics::GasType::Pollutant,
            crate::atmospherics::GasType::NitrousOxide,
            crate::atmospherics::GasType::Steam,
            crate::atmospherics::GasType::Hydrogen,
            crate::atmospherics::GasType::Water,
        ];

        let sizes = [
            FilterSize::Small,
            FilterSize::Medium,
            FilterSize::Large,
            FilterSize::Infinite,
        ];

        for &g in &gas_types {
            for &s in &sizes {
                if Filter::prefab_hash_for(g, s) == prefab_hash {
                    // Use default quantity 100 for created item
                    let mut f = Filter::new();
                    f.set_gas_type(g);
                    f.set_size(s);
                    f.set_quantity(100);
                    let filter: Shared<dyn Item> = crate::types::shared(f);
                    return Ok(WasmItem {
                        inner: Some(filter),
                    });
                }
            }
        }

        Err(JsValue::from_str(
            "Unsupported prefab hash for item creation",
        ))
    }

    /// Delegates to project-level item factory and returns a wrapper (preferred over the internal variant).
    pub fn create_item_from_prefab(prefab_hash: i32) -> Result<WasmItem, JsValue> {
        match crate::items::create_item(prefab_hash) {
            Some(shared_item) => Ok(WasmItem {
                inner: Some(shared_item),
            }),
            None => Err(JsValue::from_str(
                "Unsupported prefab hash for item creation",
            )),
        }
    }

    /// Create a base (uninitialized) filter item wrapper (not registered).
    /// Caller can set gas type/size/quantity later.
    pub fn create_base_filter() -> WasmItem {
        let filter: Shared<dyn Item> = crate::types::shared(Filter::new());
        WasmItem {
            inner: Some(filter),
        }
    }

    /// Create and register a full set of filter items (all gas types × sizes) with default quantity 100.
    /// Returns a vector of registered item IDs.
    pub fn create_all_filter_items() -> Vec<i32> {
        use crate::items::FilterSize;
        let gas_types = [
            crate::atmospherics::GasType::Oxygen,
            crate::atmospherics::GasType::Nitrogen,
            crate::atmospherics::GasType::CarbonDioxide,
            crate::atmospherics::GasType::Volatiles,
            crate::atmospherics::GasType::Pollutant,
            crate::atmospherics::GasType::NitrousOxide,
            crate::atmospherics::GasType::Steam,
            crate::atmospherics::GasType::Hydrogen,
            crate::atmospherics::GasType::Water,
        ];

        let sizes = [
            FilterSize::Small,
            FilterSize::Medium,
            FilterSize::Large,
            FilterSize::Infinite,
        ];

        let mut ids = Vec::new();
        WASM_ITEM_REGISTRY.with(|r| {
            let mut map = r.borrow_mut();
            for &g in &gas_types {
                for &s in &sizes {
                    let mut f = Filter::new();
                    f.set_gas_type(g);
                    f.set_size(s);
                    f.set_quantity(100);
                    let id = f.get_id();
                    let filter: Shared<dyn Item> = crate::types::shared(f);
                    map.insert(id, filter);
                    ids.push(id);
                }
            }
        });

        ids
    }

    pub fn get_ic(id: i32) -> Option<WasmICChip> {
        WASM_IC_REGISTRY.with(|r| {
            r.borrow()
                .get(&id)
                .cloned()
                .map(|inner| WasmICChip { inner })
        })
    }

    pub fn list_ic_ids() -> Vec<i32> {
        WASM_IC_REGISTRY.with(|r| r.borrow().keys().cloned().collect())
    }

    /// Retrieve a device by its reference id from the global registry
    pub fn get_device(id: i32) -> Option<WasmDevice> {
        WASM_DEVICE_REGISTRY
            .with(|r| r.borrow().get(&id).cloned())
            .map(|inner| WasmDevice { inner })
    }

    /// List all device ids currently in the global registry
    pub fn list_device_ids() -> Vec<i32> {
        WASM_DEVICE_REGISTRY.with(|r| r.borrow().keys().cloned().collect())
    }

    /// Remove a device from the global registry and from any network it is attached to
    pub fn remove_device(id: i32) -> Result<bool, JsValue> {
        let opt = WASM_DEVICE_REGISTRY.with(|r| r.borrow_mut().remove(&id));
        if let Some(dev) = opt {
            // If device is attached to a network, remove it
            if let Some(net) = dev.borrow().get_network() {
                let _ = net.borrow_mut().remove_device(id);
            }
            dev.borrow_mut().clear_internal_references();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
