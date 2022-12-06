#![doc = include_str!("../README.md")]

use std::{thread::sleep, time::Duration, fmt::Display, hash::Hash};

use rppal::i2c::I2c;

const IODIRA: u8 = 0x00;
const IODIRB: u8 = 0x01; 
const GPINTENA: u8 = 0x04; 
const GPINTENB: u8 = 0x05; 
const DEFVALA: u8 = 0x06; 
const DEFVALB: u8 = 0x07; 
const INTCONA: u8 = 0x08; 
const INTCONB: u8 = 0x09; 
const IOCON: u8 = 0x0A;
const GPPUA: u8 = 0x0C; 
const GPPUB: u8 = 0x0D; 
const INTFA: u8 = 0x0E; 
const INTFB: u8 = 0x0F; 
const INTCAPA: u8 = 0x10; 
const INTCAPB: u8 = 0x11; 
const GPIOA: u8 = 0x12; 
const GPIOB: u8 = 0x13; 
const OLATA: u8 = 0x14; 
const OLATB: u8 = 0x15; 

const IOCONMIRROR: Pin = Pin{pin: 6, orig: 6, shift: 0, bank: Bank::A };
const IOCONINTPOL: Pin = Pin{pin: 1, orig: 1, shift: 0, bank: Bank::A };

const NUM_GPIO: u8 = 16;

#[derive(Debug)]
/// MCP23017 Error
pub enum Error<'a> { 
    I2C(rppal::i2c::Error),
    WrongMode(&'a Pin),
    InterruptsForcedClear,
}

#[derive(Debug, Clone, Copy)]
/// Pin mode
pub enum Mode {
    Input,
    Output
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Output => f.write_str("Outpu"),
            Mode::Input => f.write_str("Input"),
        }
    }
}

impl Into<bool> for Mode {
    fn into(self) -> bool {
        match self {
            Mode::Input => true,
            Mode::Output => false,
        }
    }
}

impl From<bool> for Mode {
    fn from(s: bool) -> Self {
        if s {
            Mode::Input
        } else {
            Mode::Output
        }
    }
}


#[derive(Debug, Clone, Copy)]
/// Pin state
pub enum State {
    High,
    Low,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::High => f.write_str("High"),
            State::Low => f.write_str("Low"),
        }
    }
}

impl Into<bool> for State {
    fn into(self) -> bool {
        match self {
            State::High => true,
            State::Low => false,
        }
    }
}

impl From<bool> for State {
    fn from(s: bool) -> Self {
        if s {
            State::High
        } else {
            State::Low
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Interrupt feature state
pub enum Feature {
    On,
    Off,
}

impl Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Feature::On => f.write_str("On"),
            Feature::Off => f.write_str("Off"),
        }
    }
}

impl Into<bool> for Feature {
    fn into(self) -> bool {
        match self {
            Feature::On => true,
            Feature::Off => false,
        }
    }
}

impl From<bool> for Feature {
    fn from(s: bool) -> Self {
        if s {
            Feature::On
        } else {
            Feature::Off
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// INT pin start position
pub enum INTPOL {
    /// int pin starts high. when interrupt happens, pin goes low
    ActiveHigh,
    /// int pin starts low. when interrupt happens, pin goes high
    ActiveLow,
}

impl Display for INTPOL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            INTPOL::ActiveHigh => f.write_str("Active High"),
            INTPOL::ActiveLow => f.write_str("Active Low"),
        }
    }
}

impl Into<bool> for INTPOL {
    fn into(self) -> bool {
        match self {
            INTPOL::ActiveHigh => true,
            INTPOL::ActiveLow => false,
        }
    }
}

impl From<bool> for INTPOL {
    fn from(s: bool) -> Self {
        if s {
            INTPOL::ActiveHigh
        } else {
            INTPOL::ActiveLow
        }
    }
}


#[derive(Debug, Clone, Copy)]
/// Interrupt compare mode
pub enum Compare {
    Default,
    Previous,
}

impl Display for Compare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Compare::Default => f.write_str("Default"),
            Compare::Previous => f.write_str("Previous"),
        }
    }
}

impl Into<bool> for Compare {
    fn into(self) -> bool {
        match self {
            Compare::Default => true,
            Compare::Previous => false,
        }
    }
}

impl From<bool> for Compare {
    fn from(s: bool) -> Self {
        if s {
            Compare::Default
        } else {
            Compare::Previous
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// GPIO bank
pub enum Bank {
    A,
    B
}

impl Display for Bank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bank::A => f.write_str("A"),
            Bank::B => f.write_str("B"),
        }
    }
}

#[derive(Debug, Clone, Eq, Ord)]
/// GPIO Pin
pub struct Pin {
    pin: u8,
    orig: u8,
    shift: u8,
    bank: Bank,
}

impl Display for Pin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Pin: {} ({}), Bank: {}", self.pin, self.orig, self.bank))
    }
}

impl Pin {
    /// Pin between 0-15, 0 to 7 is bank A, 8 to 15 is bank B
    pub const fn new(pin: u8) -> Option<Pin> {
        if pin >= NUM_GPIO {
            None
        } else {
            if pin < 8 {
                Some(Pin{pin, orig: pin, shift: 0, bank: Bank::A})
            } else {
                Some(Pin{pin: pin-8, orig: pin, shift: 8, bank: Bank::B})
            }
        }
    }

    pub(crate) fn mode(&self, direction: u16) -> Mode {
        Mode::from(direction & (1 << self.orig) == 0)
    }

    pub(crate) fn apply_u16(&self, bitmap: u16, value: u8) -> u16 {
        let mut bytes = bitmap.to_be_bytes();
        if self.shift != 8 {
            bytes[1] = value.to_be_bytes()[0];
        } else {
            bytes[0] = value.to_be_bytes()[0];
        }
        return u16::from_be_bytes(bytes)
    }
}

impl PartialEq for Pin {
    fn eq(&self, other: &Self) -> bool {
        self.pin == other.pin && self.bank == other.bank
    }
}

impl PartialOrd for Pin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.bank.partial_cmp(&other.bank) {
            Some(core::cmp::Ordering::Equal) => self.pin.partial_cmp(&other.pin),
            ord => return ord,
        }
    }
}

impl Hash for Pin {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pin.hash(state);
        self.bank.hash(state);
    }
}

/// MCP23017 i2c Interface
pub struct MCP23017 {
    i2c: I2c,
    direction: u16,
    mirrored: Feature,
}

impl MCP23017 {
    pub fn new<'a>(address: u16, bus: u8) -> Result<MCP23017, Error<'a>> {
        let mut i2c = I2c::with_bus(bus).map_err(|e| Error::I2C(e))?;
        i2c.set_slave_address(address).map_err(|e| Error::I2C(e))?;

        let mut direction = i2c.smbus_read_byte(IODIRA).map_err(|e| Error::I2C(e))? as u16; 
        direction |= (i2c.smbus_read_byte(IODIRB).map_err(|e| Error::I2C(e))? as u16) << 8;

        let mcp23017 = MCP23017 { i2c, direction,  mirrored: Feature::Off};
        mcp23017.reset()?;
        Ok(mcp23017)
    }

    /// Change a specific bit in a byte
    fn change_bit(bitmap: u8, pin: &Pin, value: bool) -> u8 {
        if value {
            bitmap | (1 << pin.pin)
        } else {
            bitmap & !(1 << pin.pin)
        }
    }

    /// Set an output pin to a specific value.
    fn read_and_change_pin<'a>(&self, register: u8, pin: &'a Pin, value: bool, cur_value: Option<u8>) -> Result<u8, Error<'a>> {
        // if we don't know what the current register's full value is, get it first
        let cur_value = match cur_value {
            Some(cur_value) => cur_value,
            None => self.i2c.smbus_read_byte(register).map_err(|e| Error::I2C(e))?,
        };

        // set the single bit that corresponds to the specific pin within the full register value
        let new_value = MCP23017::change_bit(cur_value, &pin, value);

        // write and return the full register value
        self.i2c.smbus_write_byte(register, new_value).map_err(|e| Error::I2C(e))?;
        Ok(new_value)
    }

    /// Used to set the pullUp resistor setting for a pin.
    /// Returns the whole register value.
    pub fn pull_up<'a>(&self, pin: &'a Pin, value: State) -> Result<u16, Error<'a>> {
        let pull = match pin.bank {
            Bank::A => self.read_and_change_pin(GPPUA, pin, value.into(), None)?,
            Bank::B => self.read_and_change_pin(GPPUA, pin, value.into(), None)?,
        } as u16;

        Ok(pull << pin.shift)
    }

    /// Set pin to either input or output mode.
    /// Returns the value of the combined IODIRA and IODIRB registers.
    pub fn pin_mode<'a>(&mut self, pin: &'a Pin, mode: Mode) -> Result<u16, Error<'a>> {
        let mode = match pin.bank {
            Bank::A => self.read_and_change_pin(IODIRA, pin, mode.into(), None)?,
            Bank::B => self.read_and_change_pin(IODIRB, pin, mode.into(), None)?,
        };
        
        self.direction = pin.apply_u16(self.direction, mode);
        Ok(self.direction)
    }

    /// Set an output pin to a specific value.
    pub fn output<'a>(&self, pin: &'a Pin, value: State) -> Result<u8, Error<'a>>{
        if matches!(pin.mode(self.direction), Mode::Output) {
            return Err(Error::WrongMode(pin))
        }
        match pin.bank {
            Bank::A => self.read_and_change_pin(GPIOA, pin, value.into(), self.i2c.smbus_read_byte(OLATA).ok()),
            Bank::B => self.read_and_change_pin(GPIOB, pin, value.into(), self.i2c.smbus_read_byte(OLATB).ok()),
        }
    }

    /// Read the value of a pin.
    pub fn input<'a>(&self, pin: &'a Pin) -> Result<State, Error<'a>> {
        if matches!(pin.mode(self.direction), Mode::Input) {
            return Err(Error::WrongMode(pin))
        }
        
        // reads the whole register then compares the value of the specific pin
        let bank_value = match pin.bank {
            Bank::A => self.i2c.smbus_read_byte(GPIOA).map_err(|e| Error::I2C(e))?,
            Bank::B => self.i2c.smbus_read_byte(GPIOB).map_err(|e| Error::I2C(e))?,
        };

        Ok(State::from(bank_value & (1 << pin.pin) != 0))
    }


    /// Read the value of a pin regardless of it's mode
    pub fn current_val<'a>(&self, pin: &'a Pin) -> Result<State, Error<'a>> {
        // reads the whole register then compares the value of the specific pin
        let bank_value = match pin.bank {
            Bank::A => self.i2c.smbus_read_byte(GPIOA).map_err(|e| Error::I2C(e))?,
            Bank::B => self.i2c.smbus_read_byte(GPIOB).map_err(|e| Error::I2C(e))?,
        };

        Ok(State::from(bank_value & (1 << pin.pin) != 0))
    }

    /// Configure system interrupt settings.
    /// Mirror - are the int pins mirrored?
    /// Intpol - polarity of the int pin.
    pub fn config_system_interrupt<'a>(&mut self, mirror: Feature, intpol: State) -> Result<(), Error<'a>>{
        // get current register settings
        let mut register_value = self.i2c.smbus_read_byte(IOCON).map_err(|e| Error::I2C(e))?;
        // set mirror bit
        register_value = MCP23017::change_bit(register_value, &IOCONMIRROR, mirror.into());

        // set the intpol bit
        register_value = MCP23017::change_bit(register_value, &IOCONINTPOL, intpol.into());

        // set ODR pin
        self.i2c.smbus_write_byte(IOCON, register_value).map_err(|e| Error::I2C(e))?;
        self.mirrored = mirror;
        Ok(())
    }

    /// Configure interrupt setting for a specific pin. set on or off.
    pub fn config_pin_interrupt<'a>(&self, pin: &'a Pin, enabled: Feature, compare_mode: Compare, defval: Option<State>) -> Result<(), Error<'a>>{
        if matches!(pin.mode(self.direction), Mode::Input) {
            return Err(Error::WrongMode(pin))
        }

        match pin.bank {
            Bank::A => {
                // first, interrupt on change feature
                self.read_and_change_pin(GPINTENA, pin, enabled.into(), None)?;
                // then, compare mode (previous value or default value?)
                self.read_and_change_pin(INTCONA, pin, compare_mode.into(), None)?;
                // last, the default value. set it regardless if compareMode requires it, in case the requirement has changed since program start
                self.read_and_change_pin(DEFVALA, pin, defval.unwrap_or(State::Low).into(), None)?;
            },
            Bank::B => {
                self.read_and_change_pin(GPINTENB, pin, enabled.into(), None)?;
                self.read_and_change_pin(INTCONB, pin, compare_mode.into(), None)?;
                self.read_and_change_pin(DEFVALB, pin, defval.unwrap_or(State::Low).into(), None)?;

            },
        }
        Ok(())
    }

    /// Private function to return pin and value from an interrupt
    fn read_interrupt_register<'a>(&self, port: Bank) -> Result<Option<(Pin, State)>, Error<'a>> {
        match port {
            Bank::A => {
                let interrupted_a = self.i2c.smbus_read_byte(INTFA).map_err(|e| Error::I2C(e))?;
                if interrupted_a != 0 {

                    let pin = Pin::new((interrupted_a as f32).log2() as u8);
                    // get the value of the pin
                    let value_register = self.i2c.smbus_read_byte(INTCAPA).map_err(|e| Error::I2C(e))?;
                    let value = pin.clone().map(|pin| {let num = pin.pin; (pin, State::from(value_register & (1 << num) != 0))});
                    Ok(value)
                } else {
                    Ok(None)
                } 
            },
            Bank::B => {
                let interrupted_b = self.i2c.smbus_read_byte(INTFB).map_err(|e| Error::I2C(e))?;
                if interrupted_b != 0 {

                    let pin = Pin::new((interrupted_b as f32).log2() as u8);
                    // get the value of the pin
                    let value_register = self.i2c.smbus_read_byte(INTCAPB).map_err(|e| Error::I2C(e))?;
                    let value = pin.clone().map(|pin| {let num = pin.pin; (pin, State::from(value_register & (1 << num) != 0))});
                    Ok(value)
                } else {
                    Ok(None)
                } 
            }
        }
    }

    // This function should be called when INTA or INTB is triggered to indicate an interrupt occurred.
    /// The function determines the pin that caused the interrupt and gets its value.
    /// The interrupt is cleared.
    /// Returns pin and the value.
    pub fn read_interrupt<'a>(self, port: Bank) -> Result<Option<(Pin, State)>, Error<'a>> {
        // if the mirror is enabled, we don't know what port caused the interrupt, so read both
        match self.mirrored {
            Feature::On => {
                self.read_interrupt_register(Bank::A).map(|state| {
                    state.or_else(|| self.read_interrupt_register(Bank::B).unwrap_or(None))
                })
            },
            Feature::Off => self.read_interrupt_register(port),
        }
    }

    /// Check to see if there is an interrupt pending 3 times in a row (indicating it's stuck) 
    /// and if needed clear the interrupt without reading values.
    pub fn clear_interrupts<'a>(&self) -> Result<(), Error<'a>> {
        if self.i2c.smbus_read_byte(INTFA).map_err(|e| Error::I2C(e))? > 0
            || self.i2c.smbus_read_byte(INTFB).map_err(|e| Error::I2C(e))? > 0 {
            
            for _ in [0..3] {
                if self.i2c.smbus_read_byte(INTFA).map_err(|e| Error::I2C(e))? == 0
                    || self.i2c.smbus_read_byte(INTFB).map_err(|e| Error::I2C(e))? == 0 {
                    return Ok(());
                } else {
                    sleep(Duration::from_millis(500));
                }
            }

            //  force reset
            self.i2c.smbus_read_byte(GPIOA).map_err(|e| Error::I2C(e))?;
            self.i2c.smbus_read_byte(GPIOB).map_err(|e| Error::I2C(e))?;
            Err(Error::InterruptsForcedClear)
        } else {
            Ok(())
        }
    }

    /// Reset all pins and interrupts
    pub fn reset<'a>(&self) -> Result<(), Error<'a>> {
        self.i2c.smbus_write_byte(IODIRA, 0xFF).map_err(|e| Error::I2C(e))?;  // all inputs on port A
        self.i2c.smbus_write_byte(IODIRB, 0xFF).map_err(|e| Error::I2C(e))?;  // all inputs on port B
        // make sure the output registers are set to off
        self.i2c.smbus_write_byte(GPIOA, 0x00).map_err(|e| Error::I2C(e))?;
        self.i2c.smbus_write_byte(GPIOB, 0x00).map_err(|e| Error::I2C(e))?;
	    // disable the pull-ups on all ports
        self.i2c.smbus_write_byte(GPPUA, 0x00).map_err(|e| Error::I2C(e))?;
        self.i2c.smbus_write_byte(GPPUB, 0x00).map_err(|e| Error::I2C(e))?;
        // clear the IOCON configuration register, which is chip default
        self.i2c.smbus_write_byte(IOCON, 0x00).map_err(|e| Error::I2C(e))?;

        // disable interrupts on all pins 
        self.i2c.smbus_write_byte(GPINTENA, 0x00).map_err(|e| Error::I2C(e))?;
        self.i2c.smbus_write_byte(GPINTENB, 0x00).map_err(|e| Error::I2C(e))?;
        // interrupt on change register set to compare to previous value by default
        self.i2c.smbus_write_byte(INTCONA, 0x00).map_err(|e| Error::I2C(e))?;
        self.i2c.smbus_write_byte(INTCONB, 0x00).map_err(|e| Error::I2C(e))?;
        // interrupt compare value registers
        self.i2c.smbus_write_byte(DEFVALA, 0x00).map_err(|e| Error::I2C(e))?;
        self.i2c.smbus_write_byte(DEFVALB, 0x00).map_err(|e| Error::I2C(e))?;
        // clear any interrupts to start fresh
        self.i2c.smbus_read_byte(GPIOA).map_err(|e| Error::I2C(e))?;
        self.i2c.smbus_read_byte(GPIOB).map_err(|e| Error::I2C(e))?;

        Ok(())
    }
}