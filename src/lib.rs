#![no_std]

use embedded_hal::digital::{OutputPin, PinState};

pub struct AdvancedShiftRegister<const N: usize, OP: OutputPin> {
    pub shifters: [u8; N],
    data_pin: OP,
    clk_pin: OP,
    latch_pin: OP,
}

impl<const N: usize, OP: OutputPin> AdvancedShiftRegister<N, OP> {
    pub fn new(data_pin: OP, clk_pin: OP, latch_pin: OP, default_val: u8) -> Self {
        Self {
            shifters: [default_val; N],
            data_pin,
            clk_pin,
            latch_pin,
        }
    }

    pub fn get_shifter_mut(&mut self, i: usize) -> *mut u8 {
        if i >= N {
            panic!("i too large!");
        }

        core::ptr::addr_of_mut!(self.shifters[i])
    }

    pub fn shift_test(&mut self, mut val: u8, latch: bool) {
        for _ in 0..8 {
            let state = PinState::from(val & 1 > 0);
            _ = self.data_pin.set_state(state);
            val >>= 1;

            _ = self.clk_pin.set_high();
            _ = self.clk_pin.set_low();
        }

        if latch {
            _ = self.latch_pin.set_high();
            _ = self.latch_pin.set_low();
        }
    }

    pub fn shift_test_2(&mut self) {
        for i in (0..N).rev() {
            let mut val = self.shifters[i];

            for _ in 0..8 {
                let state = PinState::from(val & 1 > 0);
                _ = self.data_pin.set_state(state);
                val >>= 1;

                _ = self.clk_pin.set_high();
                _ = self.clk_pin.set_low();
            }
        }

        _ = self.latch_pin.set_high();
        _ = self.latch_pin.set_low();
    }
}