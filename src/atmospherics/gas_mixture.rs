//! Gas Mixture - container for multiple gas and liquid types
//!
//! A GasMixture holds quantities of multiple gas and liquid types and provides
//! methods for mixing, transferring, calculating properties like pressure,
//! temperature, and partial pressures, as well as phase changes.

use crate::atmospherics::{
    DEFAULT_STATE_CHANGE_RATIO, FULL_STATE_CHANGE_RATIO, MINIMUM_GAS_VOLUME,
    MINIMUM_VALID_TOTAL_MOLES, MINIMUM_WORLD_VALID_TOTAL_MOLES, PRESSURE_EQUALIZATION_EPSILON,
    calculate_energy_for_temperature_change, calculate_pressure, kelvin_to_celsius,
};

use super::{GasType, MatterState, Mole};
use std::fmt;

/// A mixture of gases and liquids with their associated energies
#[derive(Debug, Clone)]
pub struct GasMixture {
    // Gases
    pub oxygen: Mole,
    pub nitrogen: Mole,
    pub carbon_dioxide: Mole,
    pub volatiles: Mole,
    pub pollutant: Mole,
    pub nitrous_oxide: Mole,
    pub steam: Mole,
    pub hydrogen: Mole,

    // Liquids
    pub water: Mole,
    pub polluted_water: Mole,
    pub liquid_nitrogen: Mole,
    pub liquid_oxygen: Mole,
    pub liquid_volatiles: Mole,
    pub liquid_carbon_dioxide: Mole,
    pub liquid_pollutant: Mole,
    pub liquid_nitrous_oxide: Mole,
    pub liquid_hydrogen: Mole,

    /// Volume of the container in Litres
    volume: f64,
}

impl GasMixture {
    /// Create a new empty gas mixture with specified volume
    pub fn new(volume: f64) -> Self {
        Self {
            // Gases
            oxygen: Mole::zero(GasType::Oxygen),
            nitrogen: Mole::zero(GasType::Nitrogen),
            carbon_dioxide: Mole::zero(GasType::CarbonDioxide),
            volatiles: Mole::zero(GasType::Volatiles),
            pollutant: Mole::zero(GasType::Pollutant),
            nitrous_oxide: Mole::zero(GasType::NitrousOxide),
            steam: Mole::zero(GasType::Steam),
            hydrogen: Mole::zero(GasType::Hydrogen),

            // Liquids
            water: Mole::zero(GasType::Water),
            polluted_water: Mole::zero(GasType::PollutedWater),
            liquid_nitrogen: Mole::zero(GasType::LiquidNitrogen),
            liquid_oxygen: Mole::zero(GasType::LiquidOxygen),
            liquid_volatiles: Mole::zero(GasType::LiquidVolatiles),
            liquid_carbon_dioxide: Mole::zero(GasType::LiquidCarbonDioxide),
            liquid_pollutant: Mole::zero(GasType::LiquidPollutant),
            liquid_nitrous_oxide: Mole::zero(GasType::LiquidNitrousOxide),
            liquid_hydrogen: Mole::zero(GasType::LiquidHydrogen),

            volume: volume.max(MINIMUM_GAS_VOLUME),
        }
    }

    /// Get the volume in Litres
    pub fn volume(&self) -> f64 {
        self.volume
    }

    /// Set the volume in Litres
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.max(MINIMUM_GAS_VOLUME);
    }

    /// Get a reference to the mole for a specific gas type
    pub fn get_gas(&self, gas_type: GasType) -> &Mole {
        match gas_type {
            GasType::Oxygen => &self.oxygen,
            GasType::Nitrogen => &self.nitrogen,
            GasType::CarbonDioxide => &self.carbon_dioxide,
            GasType::Volatiles => &self.volatiles,
            GasType::Pollutant => &self.pollutant,
            GasType::NitrousOxide => &self.nitrous_oxide,
            GasType::Steam => &self.steam,
            GasType::Hydrogen => &self.hydrogen,
            GasType::Water => &self.water,
            GasType::PollutedWater => &self.polluted_water,
            GasType::LiquidNitrogen => &self.liquid_nitrogen,
            GasType::LiquidOxygen => &self.liquid_oxygen,
            GasType::LiquidVolatiles => &self.liquid_volatiles,
            GasType::LiquidCarbonDioxide => &self.liquid_carbon_dioxide,
            GasType::LiquidPollutant => &self.liquid_pollutant,
            GasType::LiquidNitrousOxide => &self.liquid_nitrous_oxide,
            GasType::LiquidHydrogen => &self.liquid_hydrogen,
        }
    }

    /// Get a mutable reference to the mole for a specific gas type
    pub fn get_gas_mut(&mut self, gas_type: GasType) -> &mut Mole {
        match gas_type {
            GasType::Oxygen => &mut self.oxygen,
            GasType::Nitrogen => &mut self.nitrogen,
            GasType::CarbonDioxide => &mut self.carbon_dioxide,
            GasType::Volatiles => &mut self.volatiles,
            GasType::Pollutant => &mut self.pollutant,
            GasType::NitrousOxide => &mut self.nitrous_oxide,
            GasType::Steam => &mut self.steam,
            GasType::Hydrogen => &mut self.hydrogen,
            GasType::Water => &mut self.water,
            GasType::PollutedWater => &mut self.polluted_water,
            GasType::LiquidNitrogen => &mut self.liquid_nitrogen,
            GasType::LiquidOxygen => &mut self.liquid_oxygen,
            GasType::LiquidVolatiles => &mut self.liquid_volatiles,
            GasType::LiquidCarbonDioxide => &mut self.liquid_carbon_dioxide,
            GasType::LiquidPollutant => &mut self.liquid_pollutant,
            GasType::LiquidNitrousOxide => &mut self.liquid_nitrous_oxide,
            GasType::LiquidHydrogen => &mut self.liquid_hydrogen,
        }
    }

    /// Get the quantity of a specific gas/liquid in moles
    pub fn get_moles(&self, gas_type: GasType) -> f64 {
        self.get_gas(gas_type).quantity()
    }

    /// Add gas/liquid to the mixture
    pub fn add_gas(&mut self, gas_type: GasType, moles: f64, temperature: f64) {
        let new_mole = Mole::new(gas_type, moles, temperature);
        self.get_gas_mut(gas_type).add(&new_mole);
        self.equalize_internal_energy();
        self.cleanup();
    }

    /// Add a Mole to the mixture
    pub fn add_mole(&mut self, mole: &Mole) {
        self.get_gas_mut(mole.gas_type()).add(mole);
        self.equalize_internal_energy();
        self.cleanup();
    }

    /// Add a Mole to the mixture without equalizing temperature
    pub fn add_mole_no_equalize(&mut self, mole: &Mole) {
        self.get_gas_mut(mole.gas_type()).add(mole);
    }

    /// Remove gas/liquid from the mixture, returning the removed Mole
    pub fn remove_gas(&mut self, gas_type: GasType, moles: f64) -> Mole {
        let removed = self.get_gas_mut(gas_type).remove(moles);
        self.cleanup();
        removed
    }

    /// Remove all moles of a specific gas/liquid type and return them
    pub fn remove_all_gas(&mut self, gas_type: GasType) -> Mole {
        let qty = self.get_moles(gas_type);
        if qty <= 0.0 {
            return Mole::zero(gas_type);
        }
        let removed = self.get_gas_mut(gas_type).remove(qty);
        self.cleanup();
        removed
    }

    /// Get total moles of gases only
    pub fn total_moles_gases(&self) -> f64 {
        self.oxygen.quantity()
            + self.nitrogen.quantity()
            + self.carbon_dioxide.quantity()
            + self.volatiles.quantity()
            + self.pollutant.quantity()
            + self.nitrous_oxide.quantity()
            + self.steam.quantity()
            + self.hydrogen.quantity()
    }

    /// Get total moles of liquids only
    pub fn total_moles_liquids(&self) -> f64 {
        self.water.quantity()
            + self.polluted_water.quantity()
            + self.liquid_nitrogen.quantity()
            + self.liquid_oxygen.quantity()
            + self.liquid_volatiles.quantity()
            + self.liquid_carbon_dioxide.quantity()
            + self.liquid_pollutant.quantity()
            + self.liquid_nitrous_oxide.quantity()
            + self.liquid_hydrogen.quantity()
    }

    /// Get total moles of all gases and liquids
    pub fn total_moles(&self) -> f64 {
        self.total_moles_gases() + self.total_moles_liquids()
    }

    /// Get total moles filtered by matter state
    pub fn total_moles_by_state(&self, state: MatterState) -> f64 {
        match state {
            MatterState::Gas => self.total_moles_gases(),
            MatterState::Liquid => self.total_moles_liquids(),
            MatterState::All => self.total_moles(),
            MatterState::None => 0.0,
        }
    }

    /// Get total volume of liquids in litres
    pub fn total_volume_liquids(&self) -> f64 {
        self.water.volume()
            + self.polluted_water.volume()
            + self.liquid_nitrogen.volume()
            + self.liquid_oxygen.volume()
            + self.liquid_volatiles.volume()
            + self.liquid_carbon_dioxide.volume()
            + self.liquid_pollutant.volume()
            + self.liquid_nitrous_oxide.volume()
            + self.liquid_hydrogen.volume()
    }

    /// Get the liquid volume ratio (0.0 to 1.0)
    pub fn liquid_volume_ratio(&self) -> f64 {
        if self.volume <= 0.0 {
            return 0.0;
        }
        (self.total_volume_liquids() / self.volume).min(1.0)
    }

    /// Get the available gas volume (total volume minus liquid volume)
    pub fn gas_volume(&self) -> f64 {
        (self.volume - self.total_volume_liquids()).max(MINIMUM_GAS_VOLUME)
    }

    /// Get total thermal energy in Joules (gases only)
    pub fn total_energy_gases(&self) -> f64 {
        self.oxygen.energy()
            + self.nitrogen.energy()
            + self.carbon_dioxide.energy()
            + self.volatiles.energy()
            + self.pollutant.energy()
            + self.nitrous_oxide.energy()
            + self.steam.energy()
            + self.hydrogen.energy()
    }

    /// Get total thermal energy in Joules (liquids only)
    pub fn total_energy_liquids(&self) -> f64 {
        self.water.energy()
            + self.polluted_water.energy()
            + self.liquid_nitrogen.energy()
            + self.liquid_oxygen.energy()
            + self.liquid_volatiles.energy()
            + self.liquid_carbon_dioxide.energy()
            + self.liquid_pollutant.energy()
            + self.liquid_nitrous_oxide.energy()
            + self.liquid_hydrogen.energy()
    }

    /// Get total thermal energy in Joules
    pub fn total_energy(&self) -> f64 {
        self.total_energy_gases() + self.total_energy_liquids()
    }

    /// Get total heat capacity (J/K) for gases only
    pub fn total_heat_capacity_gases(&self) -> f64 {
        self.oxygen.heat_capacity()
            + self.nitrogen.heat_capacity()
            + self.carbon_dioxide.heat_capacity()
            + self.volatiles.heat_capacity()
            + self.pollutant.heat_capacity()
            + self.nitrous_oxide.heat_capacity()
            + self.steam.heat_capacity()
            + self.hydrogen.heat_capacity()
    }

    /// Get total heat capacity (J/K) for liquids only
    pub fn total_heat_capacity_liquids(&self) -> f64 {
        self.water.heat_capacity()
            + self.polluted_water.heat_capacity()
            + self.liquid_nitrogen.heat_capacity()
            + self.liquid_oxygen.heat_capacity()
            + self.liquid_volatiles.heat_capacity()
            + self.liquid_carbon_dioxide.heat_capacity()
            + self.liquid_pollutant.heat_capacity()
            + self.liquid_nitrous_oxide.heat_capacity()
            + self.liquid_hydrogen.heat_capacity()
    }

    /// Get total heat capacity (J/K)
    pub fn total_heat_capacity(&self) -> f64 {
        self.total_heat_capacity_gases() + self.total_heat_capacity_liquids()
    }

    /// Calculate the average temperature of the mixture
    pub fn temperature(&self) -> f64 {
        let total_capacity = self.total_heat_capacity();
        if total_capacity <= 0.0 {
            return 0.0;
        }
        (self.total_energy() / total_capacity).max(0.0)
    }

    /// Calculate the gas pressure using ideal gas law (kPa)
    /// Only gases contribute to pressure, liquids don't follow ideal gas law
    pub fn pressure_gases(&self) -> f64 {
        calculate_pressure(
            self.total_moles_gases(),
            self.temperature(),
            self.gas_volume(),
        )
    }

    /// Calculate the total pressure (gases only contribute to pressure)
    pub fn pressure(&self) -> f64 {
        self.pressure_gases()
    }

    /// Calculate partial pressure for a specific gas (kPa)
    pub fn partial_pressure(&self, gas_type: GasType) -> f64 {
        if gas_type.is_liquid() {
            return 0.0; // Liquids don't contribute to partial pressure
        }
        let moles = self.get_moles(gas_type);
        calculate_pressure(moles, self.temperature(), self.gas_volume())
    }

    /// Get the ratio of a specific gas/liquid (0.0 to 1.0) relative to all content
    pub fn gas_ratio(&self, gas_type: GasType) -> f64 {
        let total = self.total_moles();
        if total <= 0.0 {
            return 0.0;
        }
        self.get_moles(gas_type) / total
    }

    /// Add energy to the mixture (distributed by heat capacity)
    pub fn add_energy(&mut self, joules: f64) {
        let total_capacity = self.total_heat_capacity();
        if total_capacity <= 0.0 {
            return;
        }

        // Distribute energy to all gases and liquids
        for gas_type in GasType::all() {
            let gas = self.get_gas_mut(gas_type);
            let ratio = gas.heat_capacity() / total_capacity;
            gas.add_energy(joules * ratio);
        }
    }

    /// Remove energy from the mixture
    /// Returns the actual amount removed
    pub fn remove_energy(&mut self, joules: f64) -> f64 {
        let total_energy = self.total_energy();
        let to_remove = joules.min(total_energy);

        if to_remove <= 0.0 {
            return 0.0;
        }

        let total_capacity = self.total_heat_capacity();
        let mut removed = 0.0;

        for gas_type in GasType::all() {
            let gas = self.get_gas_mut(gas_type);
            let ratio = gas.heat_capacity() / total_capacity;
            removed += gas.remove_energy(to_remove * ratio);
        }

        removed
    }

    /// Set the temperature of all gases and liquids
    pub fn set_temperature(&mut self, temperature: f64) {
        for gas_type in GasType::all() {
            self.get_gas_mut(gas_type).set_temperature(temperature);
        }
    }

    /// Equalize internal energy (set all components to the same temperature)
    pub fn equalize_internal_energy(&mut self) {
        let temp = self.temperature();
        self.set_temperature(temp);
    }

    // ===== Phase Changes =====

    /// Process phase changes for all gases and liquids
    /// Returns the number of phase changes that occurred
    pub fn process_phase_changes(&mut self) -> u32 {
        let pressure = self.pressure();
        let volume = self.gas_volume();
        let mut changes = 0u32;

        // Collect phase change results
        let mut additions: Vec<Mole> = Vec::new();

        // Process all gases and liquids
        for gas_type in GasType::all() {
            let gas = self.get_gas_mut(gas_type);
            let result = gas.change_state(pressure, volume, 0.0, true);
            if result.occurred
                && let Some(changed) = result.changed
            {
                additions.push(changed);
                changes += 1;
            }
        }

        // Add the changed moles to the mixture
        for mole in additions {
            self.add_mole_no_equalize(&mole);
        }

        // Equalize temperature after all phase changes
        if changes > 0 {
            self.equalize_internal_energy();
        }

        self.cleanup();
        changes
    }

    // ===== Transfer Methods =====

    /// Transfer a ratio of all gases to another mixture
    /// Returns the total moles transferred
    pub fn transfer_ratio_to(
        &mut self,
        target: &mut GasMixture,
        ratio: f64,
        state: MatterState,
    ) -> f64 {
        let ratio = ratio.clamp(0.0, 1.0);
        let mut transferred = 0.0;

        for gas_type in GasType::all() {
            if gas_type.matches_state(state) {
                let removed = self.get_gas_mut(gas_type).remove_ratio(ratio);
                transferred += removed.quantity();
                target.get_gas_mut(gas_type).add(&removed);
            }
        }

        self.cleanup();
        target.cleanup();

        transferred
    }

    /// Transfer gas by pressure difference
    /// Moves gas from higher to lower pressure until equilibrium
    pub fn equalize_with(&mut self, other: &mut GasMixture) {
        // First, equalize temperatures by transferring thermal energy
        let self_heat_capacity = self.total_heat_capacity();
        let other_heat_capacity = other.total_heat_capacity();
        let total_heat_capacity = self_heat_capacity + other_heat_capacity;

        if total_heat_capacity > 0.0 {
            let total_energy = self.total_energy() + other.total_energy();
            let equilibrium_temp = total_energy / total_heat_capacity;

            self.set_temperature(equilibrium_temp);
            other.set_temperature(equilibrium_temp);
        }

        // Then, equalize gas pressures
        let self_pressure = self.pressure();
        let other_pressure = other.pressure();

        if (self_pressure - other_pressure).abs() < PRESSURE_EQUALIZATION_EPSILON {
            return;
        }

        // Calculate the amount to transfer to reach equilibrium
        let total_moles = self.total_moles_gases() + other.total_moles_gases();
        let total_volume = self.gas_volume() + other.gas_volume();

        // Target moles for each based on volume ratio
        let self_target = total_moles * (self.gas_volume() / total_volume);

        let self_current = self.total_moles_gases();

        if self_current > self_target {
            // Transfer from self to other
            let to_transfer = self_current - self_target;
            let ratio = to_transfer / self_current;
            self.transfer_ratio_to(other, ratio, MatterState::Gas);
        } else {
            // Transfer from other to self
            let other_current = other.total_moles_gases();
            let to_transfer = other_current - (total_moles - self_target);
            if other_current > 0.0 {
                let ratio = to_transfer / other_current;
                other.transfer_ratio_to(self, ratio, MatterState::Gas);
            }
        }

        self.cleanup();
        other.cleanup();
    }

    /// Remove a specific amount of moles, proportionally from gases/liquids
    /// Returns a new GasMixture with the removed content
    pub fn remove_moles(&mut self, moles: f64, state: MatterState) -> GasMixture {
        let total = self.total_moles_by_state(state);
        if total <= 0.0 || moles <= 0.0 {
            return GasMixture::new(0.0);
        }

        let ratio = (moles / total).min(1.0);
        let mut removed = GasMixture::new(0.0);

        for gas_type in GasType::all() {
            if gas_type.matches_state(state) {
                let gas = self.get_gas_mut(gas_type).remove_ratio(ratio);
                removed.get_gas_mut(gas_type).add(&gas);
            }
        }

        self.cleanup();
        removed
    }

    /// Merge another gas mixture into this one
    pub fn merge(&mut self, other: &GasMixture) {
        for gas_type in GasType::all() {
            self.get_gas_mut(gas_type).add(other.get_gas(gas_type));
        }
        self.cleanup();
    }

    /// Merge another gas mixture into this one (by state)
    pub fn merge_by_state(&mut self, other: &GasMixture, state: MatterState) {
        for gas_type in GasType::all() {
            if gas_type.matches_state(state) {
                self.get_gas_mut(gas_type).add(other.get_gas(gas_type));
            }
        }
        self.cleanup();
    }

    /// Clear all gases and liquids
    pub fn clear(&mut self) {
        for gas_type in GasType::all() {
            self.get_gas_mut(gas_type).clear();
        }
    }

    /// Clear only gases
    pub fn clear_gases(&mut self) {
        for gas_type in GasType::all_gases() {
            self.get_gas_mut(gas_type).clear();
        }
    }

    /// Clear only liquids
    pub fn clear_liquids(&mut self) {
        for gas_type in GasType::all_liquids() {
            self.get_gas_mut(gas_type).clear();
        }
    }

    /// Clean up tiny residues to avoid asymptotic non-zero leftovers
    pub fn cleanup(&mut self) {
        if self.total_moles() < MINIMUM_VALID_TOTAL_MOLES {
            self.clear();
        }
    }

    /// Check if the mixture is empty
    pub fn is_empty(&self) -> bool {
        self.total_moles() < MINIMUM_VALID_TOTAL_MOLES
    }

    /// Scale all contents by a ratio
    pub fn scale(&mut self, ratio: f64, state: MatterState) {
        for gas_type in GasType::all() {
            if gas_type.matches_state(state) {
                self.get_gas_mut(gas_type).scale(ratio);
            }
        }
        self.cleanup();
    }
}

impl fmt::Display for GasMixture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "GasMixture ({:.1} L, {:.1} L gas volume):",
            self.volume,
            self.gas_volume()
        )?;
        writeln!(
            f,
            "  Temperature: {:.4} K ({:.4} °C)",
            self.temperature(),
            kelvin_to_celsius(self.temperature())
        )?;
        writeln!(f, "  Pressure: {:.4} kPa", self.pressure())?;
        writeln!(
            f,
            "  Total Moles: {:.4} (Gas: {:.4}, Liquid: {:.4})",
            self.total_moles(),
            self.total_moles_gases(),
            self.total_moles_liquids()
        )?;

        // List gases
        let has_gases = self.total_moles_gases() > 0.0;
        if has_gases {
            writeln!(f, "  Gases:")?;
            for gas_type in GasType::all_gases() {
                let mole = self.get_gas(gas_type);
                if !mole.is_empty() {
                    writeln!(
                        f,
                        "    {}: {:.4} mol ({:.1}%)",
                        gas_type.symbol(),
                        mole.quantity(),
                        self.gas_ratio(gas_type) * 100.0
                    )?;
                }
            }
        }

        // List liquids
        let has_liquids = self.total_moles_liquids() > 0.0;
        if has_liquids {
            writeln!(f, "  Liquids ({:.4} L):", self.total_volume_liquids())?;
            for gas_type in GasType::all_liquids() {
                let mole = self.get_gas(gas_type);
                if !mole.is_empty() {
                    writeln!(
                        f,
                        "    {}: {:.4} mol ({:.4} L)",
                        gas_type.symbol(),
                        mole.quantity(),
                        mole.volume()
                    )?;
                }
            }
        }

        // Calculate latent energy that will be moved next tick and stored latent heat
        // Positive = energy available to evaporate liquids; Negative = energy released by condensation
        let mut latent_energy: f64 = 0.0;
        let mut stored_latent: f64 = 0.0; // total latent heat stored in liquids (n * L)
        let pressure = self.pressure();

        writeln!(f, "  Phase change breakdown:")?;
        for gas_type in GasType::all() {
            let mole = self.get_gas(gas_type);
            if mole.is_empty() {
                continue;
            }

            if gas_type.is_liquid() {
                let evap_temp = mole.evaporation_temperature_clamped(pressure);

                let ratio = if mole.quantity() <= MINIMUM_WORLD_VALID_TOTAL_MOLES {
                    FULL_STATE_CHANGE_RATIO
                } else {
                    DEFAULT_STATE_CHANGE_RATIO
                };
                let energy_next_tick =
                    mole.latent_energy_next_tick(pressure, self.volume(), 0.0, true, ratio);
                let stored = mole.quantity() * gas_type.latent_heat_of_vaporization();
                stored_latent += stored;

                writeln!(
                    f,
                    "    {} (liquid): {:.4} mol @ {:.2} K -> evap_temp {:.2} K: next_tick {:.4} J, stored latent {:.4} J",
                    gas_type.symbol(),
                    mole.quantity(),
                    mole.temperature(),
                    evap_temp,
                    energy_next_tick,
                    stored
                )?;

                if energy_next_tick > 0.0 {
                    latent_energy += energy_next_tick;
                }
            } else if gas_type.is_gas() {
                let cond_temp = mole.evaporation_temperature_clamped(pressure);
                let deficit_energy = calculate_energy_for_temperature_change(
                    mole.quantity(),
                    gas_type.specific_heat(),
                    cond_temp - mole.temperature(),
                );

                writeln!(
                    f,
                    "    {} (gas): {:.4} mol @ {:.2} K -> cond_temp {:.2} K: deficit {:.4} J",
                    gas_type.symbol(),
                    mole.quantity(),
                    mole.temperature(),
                    cond_temp,
                    deficit_energy
                )?;

                if deficit_energy > 0.0 {
                    latent_energy -= deficit_energy; // condensation would release this energy
                }
            }
        }

        // Compact energy summary (single-line for quick comparison)
        writeln!(
            f,
            "  Energies: total: {:.2} J, gases={:.2} J, liquids={:.2} J",
            self.total_energy(),
            self.total_energy_gases(),
            self.total_energy_liquids(),
        )?;

        writeln!(
            f,
            "  Latent: stored_latent={:.2} J, latent_potential={:+.2} J, latent_rate={:+.2} J",
            stored_latent, latent_energy, -latent_energy
        )?;

        Ok(())
    }
}
