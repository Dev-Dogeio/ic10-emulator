//! Mole struct for gas quantities with energy tracking
//!
//! A Mole represents a quantity of a specific gas type along with its
//! thermal energy. This allows for accurate temperature and energy
//! calculations during gas transfers and mixing. Supports phase changes
//! between gas and liquid states.

use crate::conversions::lerp;

use super::{GasType, MatterState, chemistry};
use std::fmt;

/// Result of a phase change operation
#[derive(Debug, Clone, Copy)]
pub struct PhaseChangeResult {
    /// The changed mole (in the new state)
    pub changed: Option<Mole>,
    /// Whether the phase change occurred
    pub occurred: bool,
}

impl PhaseChangeResult {
    /// No phase change occurred
    pub fn none() -> Self {
        Self {
            changed: None,
            occurred: false,
        }
    }

    /// Phase change occurred
    pub fn some(mole: Mole) -> Self {
        Self {
            changed: Some(mole),
            occurred: true,
        }
    }
}

/// Represents a quantity of gas with its associated thermal energy
#[derive(Debug, Clone, Copy)]
pub struct Mole {
    /// The type of gas
    gas_type: GasType,
    /// Quantity of gas in moles
    quantity: f64,
    /// Thermal energy in Joules
    energy: f64,
}

impl Mole {
    /// Create a new Mole with specified quantity and temperature
    ///
    /// # Arguments
    /// * `gas_type` - The type of gas
    /// * `quantity` - Amount in moles
    /// * `temperature` - Temperature in Kelvin
    pub fn new(gas_type: GasType, quantity: f64, temperature: f64) -> Self {
        let quantity = quantity.max(0.0);
        let temperature = temperature.max(0.0);
        let energy = quantity * gas_type.specific_heat() * temperature;

        Self {
            gas_type,
            quantity,
            energy,
        }
    }

    /// Create a new Mole with zero quantity
    pub fn zero(gas_type: GasType) -> Self {
        Self {
            gas_type,
            quantity: 0.0,
            energy: 0.0,
        }
    }

    /// Create a Mole with specified quantity and energy directly
    pub fn with_energy(gas_type: GasType, quantity: f64, energy: f64) -> Self {
        Self {
            gas_type,
            quantity: quantity.max(0.0),
            energy: energy.max(0.0),
        }
    }

    /// Get the gas type
    pub fn gas_type(&self) -> GasType {
        self.gas_type
    }

    /// Get the quantity in moles
    pub fn quantity(&self) -> f64 {
        self.quantity
    }

    /// Get the thermal energy in Joules
    pub fn energy(&self) -> f64 {
        self.energy
    }

    /// Calculate the temperature in Kelvin
    /// T = E / (n * Cv)
    pub fn temperature(&self) -> f64 {
        if self.quantity <= chemistry::MINIMUM_QUANTITY_MOLES {
            return 0.0;
        }
        let temp = self.energy / (self.quantity * self.gas_type.specific_heat());
        temp.max(0.0)
    }

    /// Get the heat capacity of this gas amount (J/K)
    /// C = n * Cv
    pub fn heat_capacity(&self) -> f64 {
        self.quantity * self.gas_type.specific_heat()
    }

    /// Check if this mole is effectively empty
    pub fn is_empty(&self) -> bool {
        self.quantity < chemistry::MINIMUM_QUANTITY_MOLES
    }

    /// Set the quantity, adjusting energy to maintain temperature
    pub fn set_quantity(&mut self, new_quantity: f64) {
        let temp = self.temperature();
        self.quantity = new_quantity.max(0.0);
        self.energy = self.quantity * self.gas_type.specific_heat() * temp;
        self.cleanup();
    }

    /// Set the temperature, adjusting energy accordingly
    pub fn set_temperature(&mut self, temperature: f64) {
        let temp = temperature.max(0.0);
        self.energy = self.quantity * self.gas_type.specific_heat() * temp;
    }

    /// Add energy (heating)
    pub fn add_energy(&mut self, joules: f64) {
        self.energy = (self.energy + joules).max(0.0);
    }

    /// Remove energy (cooling)
    /// Returns the actual amount removed
    pub fn remove_energy(&mut self, joules: f64) -> f64 {
        let removed = joules.min(self.energy);
        self.energy -= removed;
        removed
    }

    /// Add moles from another Mole of the same gas type
    /// Combines quantities and energies
    ///
    /// # Panics
    /// Panics if gas types don't match
    pub fn add(&mut self, other: &Mole) {
        assert_eq!(
            self.gas_type, other.gas_type,
            "Cannot add moles of different gas types"
        );
        self.quantity += other.quantity;
        self.energy += other.energy;
        self.cleanup();
    }

    /// Remove a specified quantity, returning the removed Mole
    /// Energy is proportionally removed
    ///
    /// # Arguments
    /// * `amount` - Amount in moles to remove
    ///
    /// # Returns
    /// A new Mole containing the removed gas
    pub fn remove(&mut self, amount: f64) -> Mole {
        let amount = amount.min(self.quantity).max(0.0);
        if amount <= 0.0 || self.quantity <= 0.0 {
            return Mole::zero(self.gas_type);
        }

        let removed_energy = self.energy * amount / self.quantity;

        self.quantity -= amount;
        self.energy -= removed_energy;

        self.cleanup();
        Mole::with_energy(self.gas_type, amount, removed_energy)
    }

    /// Remove a ratio of the gas (0.0 to 1.0)
    /// Returns the removed Mole
    pub fn remove_ratio(&mut self, ratio: f64) -> Mole {
        self.remove(self.quantity * ratio.clamp(0.0, 1.0))
    }

    /// Transfer gas to another Mole of the same type
    /// Returns the amount actually transferred
    pub fn transfer_to(&mut self, target: &mut Mole, amount: f64) -> f64 {
        let removed = self.remove(amount);
        let transferred = removed.quantity;
        target.add(&removed);
        transferred
    }

    /// Calculate the energy required to reach a target temperature
    pub fn energy_to_reach_temperature(&self, target_temp: f64) -> f64 {
        let current_temp = self.temperature();
        let delta_temp = target_temp - current_temp;
        self.heat_capacity() * delta_temp
    }

    /// Clear this mole (set to zero)
    pub fn clear(&mut self) {
        self.quantity = 0.0;
        self.energy = 0.0;
    }

    /// Cleanup small residues: zero out quantity and energy if below threshold
    pub fn cleanup(&mut self) {
        if self.quantity < chemistry::MINIMUM_QUANTITY_MOLES {
            self.clear();
        }
    }

    /// Get the matter state of this mole
    pub fn matter_state(&self) -> MatterState {
        self.gas_type.matter_state()
    }

    /// Check if this mole will freeze at its current temperature
    pub fn will_freeze(&self) -> bool {
        if self.quantity < chemistry::MINIMUM_QUANTITY_MOLES {
            return false;
        }
        self.temperature() <= self.gas_type.freezing_temperature()
    }

    /// Get the volume of this liquid in litres
    /// Returns 0 for gases
    pub fn volume(&self) -> f64 {
        self.gas_type.molar_volume() * self.quantity
    }

    /// Get the mass of this gas/liquid in grams
    pub fn mass(&self) -> f64 {
        self.gas_type.molar_mass() * self.quantity
    }

    /// Calculate the evaporation temperature for a given pressure
    /// This is clamped between the freezing and max liquid temperatures
    pub fn evaporation_temperature_clamped(&self, pressure: f64) -> f64 {
        let clamped_pressure = pressure.clamp(
            self.gas_type.min_liquid_pressure(),
            self.gas_type.critical_pressure(),
        );

        self.calculate_evaporation_temperature(clamped_pressure)
            .clamp(
                self.gas_type.freezing_temperature(),
                self.gas_type.max_liquid_temperature(),
            )
    }

    /// Calculate the evaporation pressure for a given temperature
    /// This is clamped between min liquid pressure and critical pressure
    pub fn evaporation_pressure_clamped(&self, temperature: f64) -> f64 {
        let clamped_temp = temperature.clamp(
            self.gas_type.freezing_temperature(),
            self.gas_type.max_liquid_temperature(),
        );

        self.calculate_evaporation_pressure(clamped_temp).clamp(
            self.gas_type.min_liquid_pressure(),
            self.gas_type.critical_pressure(),
        )
    }

    /// Calculate evaporation temperature from pressure using power law formula
    /// T = (P / A)^(1/B) where P is pressure in kPa
    fn calculate_evaporation_temperature(&self, pressure: f64) -> f64 {
        let a = self.gas_type.evaporation_coefficient_a();
        let b = self.gas_type.evaporation_coefficient_b();
        (pressure / a).powf(1.0 / b)
    }

    /// Calculate evaporation pressure from temperature using power law formula
    /// P = A * T^B where T is temperature in K
    fn calculate_evaporation_pressure(&self, temperature: f64) -> f64 {
        let a = self.gas_type.evaporation_coefficient_a();
        let b = self.gas_type.evaporation_coefficient_b();
        a * temperature.powf(b)
    }

    /// Compute the core evaporation state (final energy available and max quantity)
    fn compute_evaporation_base(
        &self,
        pressure: f64,
        volume: f64,
        temperature_offset: f64,
        prevent_absolute_zero_evaporation: bool,
    ) -> Option<(f64, f64)> {
        if !self.gas_type.can_evaporate() {
            return None;
        }

        self.gas_type.evaporation_type()?;

        let evap_temp = self.evaporation_temperature_clamped(pressure) + temperature_offset;
        let mut evap_pressure = self.evaporation_pressure_clamped(self.temperature());

        // Calculate minimum evaporation temperature (half of freezing point)
        let half_freezing = self.gas_type.freezing_temperature() * chemistry::HALF_FREEZING_FACTOR;

        // Adjust evaporation pressure for sub-freezing temperatures
        if self.temperature() < self.gas_type.freezing_temperature()
            && prevent_absolute_zero_evaporation
        {
            let effective_temp = self.temperature().max(half_freezing);
            evap_pressure = chemistry::map_to_scale(
                half_freezing,
                self.gas_type.freezing_temperature(),
                chemistry::ARMSTRONG_LIMIT,
                self.gas_type.min_liquid_pressure(),
                effective_temp,
            );
        }

        let pressure_delta = evap_pressure - pressure;

        // Determine effective evaporation temperature
        let effective_evap_temp = if self.temperature() <= evap_temp {
            if pressure > evap_pressure || self.temperature() <= half_freezing {
                return None;
            }
            // Interpolate evaporation temperature based on pressure
            let t = (pressure_delta / self.gas_type.min_liquid_pressure()).clamp(0.0, 1.0);
            lerp(
                self.temperature(),
                self.temperature() - chemistry::EVAP_INTERPOLATION_TEMP_DELTA,
                t,
            )
            .max(half_freezing)
        } else {
            evap_temp
        };

        let mut max_quantity = self.quantity;
        if pressure < evap_pressure {
            max_quantity =
                chemistry::calculate_moles(pressure_delta, volume, self.temperature()).max(0.0);
        }
        max_quantity = max_quantity.min(self.quantity);

        if max_quantity <= 0.0 {
            return None;
        }

        // Calculate the energy available for state change
        let energy_for_change = chemistry::calculate_energy_for_temperature_change(
            self.quantity,
            self.gas_type.specific_heat(),
            self.temperature() - effective_evap_temp,
        );

        // Handle near-freezing conditions with rate limiting (time-limited energy)
        let adjusted_energy = if self.temperature()
            < self.gas_type.freezing_temperature() + chemistry::NEAR_FREEZING_MARGIN
            && self.temperature() > half_freezing + chemistry::NEAR_FREEZING_MARGIN
            && prevent_absolute_zero_evaporation
        {
            chemistry::SMALL_STATE_CHANGE_RATE * self.gas_type.specific_heat() * self.quantity
        } else {
            energy_for_change
        };

        // Force full evaporation if above critical temperature
        let final_energy = if self.temperature() > self.gas_type.max_liquid_temperature() {
            let min_energy = chemistry::calculate_energy_for_temperature_change(
                self.quantity,
                self.gas_type.specific_heat(),
                self.temperature() - self.gas_type.max_liquid_temperature(),
            );
            adjusted_energy.max(min_energy)
        } else {
            adjusted_energy
        };

        Some((final_energy, max_quantity))
    }

    /// Compute the latent energy that will be moved on the *next* simulation tick (J).
    pub fn latent_energy_next_tick(
        &self,
        pressure: f64,
        volume: f64,
        temperature_offset: f64,
        prevent_absolute_zero_evaporation: bool,
        ratio: f64,
    ) -> f64 {
        let base = match self.compute_evaporation_base(
            pressure,
            volume,
            temperature_offset,
            prevent_absolute_zero_evaporation,
        ) {
            Some(b) => b,
            None => return 0.0,
        };

        let (final_energy, max_quantity) = base;

        // Minimum energy for a meaningful state change
        let latent_heat = self.gas_type.latent_heat_of_vaporization();
        let min_change_energy = chemistry::MINIMUM_QUANTITY_MOLES * latent_heat;

        // If not enough energy for a meaningful change, only allow it for very small quantities
        let mut ratio = ratio;
        if final_energy < min_change_energy {
            if self.quantity < chemistry::MINIMUM_WORLD_VALID_TOTAL_MOLES {
                ratio = chemistry::FULL_STATE_CHANGE_RATIO;
            } else {
                return 0.0;
            }
        }

        let moles_to_change =
            chemistry::calculate_moles_for_state_change(final_energy, latent_heat);
        let scale = if moles_to_change > max_quantity {
            moles_to_change / max_quantity
        } else {
            1.0
        };
        let effective_ratio = if moles_to_change < chemistry::LOW_STATE_CHANGE_QUANTITY_BOUND {
            chemistry::SMALL_STATE_CHANGE_RATE
        } else {
            ratio
        };

        // Energy that will be consumed this tick
        (final_energy / scale) * effective_ratio
    }

    /// Perform phase change based on current conditions
    pub fn change_state(
        &mut self,
        pressure: f64,
        volume: f64,
        temperature_offset: f64,
        prevent_absolute_zero_evaporation: bool,
    ) -> PhaseChangeResult {
        if self.quantity < chemistry::MINIMUM_QUANTITY_MOLES {
            return PhaseChangeResult::none();
        }

        match self.matter_state() {
            MatterState::Liquid => self.try_evaporate(
                pressure,
                volume,
                temperature_offset,
                prevent_absolute_zero_evaporation,
            ),
            MatterState::Gas => self.try_condense(pressure, temperature_offset),
            _ => PhaseChangeResult::none(),
        }
    }

    /// Try to evaporate liquid into gas
    fn try_evaporate(
        &mut self,
        pressure: f64,
        volume: f64,
        temperature_offset: f64,
        prevent_absolute_zero_evaporation: bool,
    ) -> PhaseChangeResult {
        if !self.gas_type.can_evaporate() {
            return PhaseChangeResult::none();
        }

        let evap_type = match self.gas_type.evaporation_type() {
            Some(t) => t,
            None => return PhaseChangeResult::none(),
        };

        let base = match self.compute_evaporation_base(
            pressure,
            volume,
            temperature_offset,
            prevent_absolute_zero_evaporation,
        ) {
            Some(b) => b,
            None => return PhaseChangeResult::none(),
        };

        let (final_energy, final_max_quantity) = base;

        // Check if there's enough energy for meaningful state change
        let min_change_energy =
            chemistry::MINIMUM_QUANTITY_MOLES * self.gas_type.latent_heat_of_vaporization();

        if final_energy < min_change_energy {
            if self.quantity < chemistry::MINIMUM_WORLD_VALID_TOTAL_MOLES {
                return self.state_change_liquid(
                    final_energy,
                    chemistry::FULL_STATE_CHANGE_RATIO,
                    final_max_quantity,
                    evap_type,
                );
            }
            return PhaseChangeResult::none();
        }

        self.state_change_liquid(
            final_energy,
            chemistry::DEFAULT_STATE_CHANGE_RATIO,
            final_max_quantity,
            evap_type,
        )
    }

    /// Try to condense gas into liquid
    fn try_condense(&mut self, pressure: f64, temperature_offset: f64) -> PhaseChangeResult {
        if !self.gas_type.can_condense() {
            return PhaseChangeResult::none();
        }

        if pressure < self.gas_type.min_liquid_pressure() {
            return PhaseChangeResult::none();
        }

        let condensation_type = match self.gas_type.condensation_type() {
            Some(t) => t,
            None => return PhaseChangeResult::none(),
        };

        let condensation_temp = self.evaporation_temperature_clamped(pressure) + temperature_offset;

        if self.temperature() >= condensation_temp {
            return PhaseChangeResult::none();
        }

        // Calculate energy deficit (how much energy needs to be added to reach condensation temp)
        let deficit_energy = chemistry::calculate_energy_for_temperature_change(
            self.quantity,
            self.gas_type.specific_heat(),
            condensation_temp - self.temperature(),
        );

        let min_change_energy =
            chemistry::MINIMUM_QUANTITY_MOLES * self.gas_type.latent_heat_of_vaporization();

        if deficit_energy < min_change_energy {
            if self.quantity <= chemistry::MINIMUM_WORLD_VALID_TOTAL_MOLES {
                return self.state_change_gas(
                    deficit_energy,
                    chemistry::FULL_STATE_CHANGE_RATIO,
                    condensation_type,
                );
            }
            return PhaseChangeResult::none();
        }

        self.state_change_gas(
            deficit_energy,
            chemistry::DEFAULT_STATE_CHANGE_RATIO,
            condensation_type,
        )
    }

    /// Perform liquid -> gas state change
    fn state_change_liquid(
        &mut self,
        energy_for_change: f64,
        ratio: f64,
        max_quantity: f64,
        target_type: GasType,
    ) -> PhaseChangeResult {
        if energy_for_change <= 0.0 {
            return PhaseChangeResult::none();
        }

        let max_quantity = max_quantity.min(self.quantity);
        let latent_heat = self.gas_type.latent_heat_of_vaporization();

        // Calculate how many moles can change state
        let moles_to_change =
            chemistry::calculate_moles_for_state_change(energy_for_change, latent_heat);

        // Limit to available quantity
        let scale = if moles_to_change > max_quantity {
            moles_to_change / max_quantity
        } else {
            1.0
        };

        // Apply rate limiting for small quantities
        let effective_ratio = if moles_to_change < chemistry::LOW_STATE_CHANGE_QUANTITY_BOUND {
            chemistry::SMALL_STATE_CHANGE_RATE
        } else {
            ratio
        };

        // Scale quantities
        let moles_to_change = (moles_to_change / scale) * effective_ratio;
        let energy_used = (energy_for_change / scale) * effective_ratio;

        // Calculate energy to transfer with the changed moles
        let energy_fraction = moles_to_change / self.quantity;
        let energy_to_transfer = self.energy * energy_fraction;

        // Update this mole
        let remaining_energy = self.energy - energy_to_transfer - energy_used;
        let remaining_quantity = (self.quantity - moles_to_change).max(0.0);

        // Handle energy underflow
        let (final_remaining_energy, extra_deficit) = if remaining_energy < 0.0 {
            (0.0, remaining_energy)
        } else {
            (remaining_energy, 0.0)
        };

        // Safety check: don't leave energy without quantity or vice versa
        if remaining_quantity > 0.0 && final_remaining_energy <= 0.0 {
            return PhaseChangeResult::none();
        }
        if remaining_quantity <= 0.0 && final_remaining_energy > 0.0 {
            return PhaseChangeResult::none();
        }

        self.quantity = remaining_quantity;
        self.energy = final_remaining_energy;
        self.cleanup();

        // Create the evaporated gas
        let changed = Mole::with_energy(
            target_type,
            moles_to_change,
            energy_to_transfer + extra_deficit,
        );

        PhaseChangeResult::some(changed)
    }

    /// Perform gas -> liquid state change
    fn state_change_gas(
        &mut self,
        deficit_energy: f64,
        ratio: f64,
        target_type: GasType,
    ) -> PhaseChangeResult {
        let latent_heat = self.gas_type.latent_heat_of_vaporization();

        // Limit energy to what's available
        let max_energy = chemistry::calculate_energy_for_state_change(self.quantity, latent_heat);
        let deficit_energy = deficit_energy.min(max_energy);

        // Calculate moles to change
        let moles_to_change =
            chemistry::calculate_moles_for_state_change(deficit_energy, latent_heat);

        // Limit to available quantity
        let scale = if moles_to_change > self.quantity {
            moles_to_change / self.quantity
        } else {
            1.0
        };

        // Apply rate limiting for small quantities
        let effective_ratio = if moles_to_change < chemistry::LOW_STATE_CHANGE_QUANTITY_BOUND {
            chemistry::SMALL_STATE_CHANGE_RATE
        } else {
            ratio
        };

        // Scale quantities
        let moles_to_change = (moles_to_change / scale) * effective_ratio;
        let energy_released = (deficit_energy / scale) * effective_ratio;

        // Calculate energy to transfer with the changed moles
        let energy_fraction = moles_to_change / self.quantity;
        let energy_to_transfer = self.energy * energy_fraction;

        // Update this mole - condensation releases energy
        let remaining_energy = self.energy - energy_to_transfer + energy_released;
        let remaining_quantity = (self.quantity - moles_to_change).max(0.0);

        // Handle energy underflow
        let (final_remaining_energy, extra_deficit) = if remaining_energy < 0.0 {
            (0.0, remaining_energy)
        } else {
            (remaining_energy, 0.0)
        };

        self.quantity = remaining_quantity;
        self.energy = final_remaining_energy;
        self.cleanup();

        // Create the condensed liquid
        let changed = Mole::with_energy(
            target_type,
            moles_to_change,
            energy_to_transfer + extra_deficit,
        );

        PhaseChangeResult::some(changed)
    }

    /// Set quantity and energy directly
    pub fn set(&mut self, quantity: f64, energy: f64) {
        self.quantity = if quantity < 0.0 || quantity.is_nan() {
            0.0
        } else {
            quantity
        };
        self.energy = if energy < 0.0 || energy.is_nan() {
            0.0
        } else {
            energy
        };
        self.cleanup();
    }

    /// Scale quantity and energy by a factor
    pub fn scale(&mut self, factor: f64) {
        let factor = if factor.is_nan() || factor < 0.0 {
            0.0
        } else {
            factor
        };
        self.quantity *= factor;
        self.energy *= factor;
        self.cleanup();
    }
}

impl Default for Mole {
    fn default() -> Self {
        Self {
            gas_type: GasType::Nitrogen,
            quantity: 0.0,
            energy: 0.0,
        }
    }
}

impl fmt::Display for Mole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {:.4} mol @ {:.2} K",
            self.gas_type.symbol(),
            self.quantity,
            self.temperature()
        )
    }
}
