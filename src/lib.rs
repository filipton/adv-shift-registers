#![no_std]

use core::{borrow::BorrowMut, ops::Range};
use embedded_hal::digital::{ErrorType, OutputPin, PinState};

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

    pub fn get_shifter_mut(&mut self, i: usize) -> ShifterValue {
        ShifterValue {
            inner: core::ptr::addr_of_mut!(self.shifters[i]),
        }
    }

    pub fn get_shifter_range_mut(&mut self, range: Range<usize>) -> ShifterValueRange {
        ShifterValueRange {
            //len: range.len(),
            inner: core::ptr::addr_of_mut!(self.shifters[range]),
        }
    }

    pub fn update_shifters(&mut self) {
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

#[derive(Clone)]
pub struct ShifterValue {
    inner: *mut u8,
}

impl ShifterValue {
    pub fn set_value(&self, value: u8) {
        unsafe {
            *self.inner = value;
        }
    }

    pub fn value<'a>(&self) -> &'a mut u8 {
        unsafe { self.inner.as_mut().unwrap() }
    }
}

#[derive(Clone)]
pub struct ShifterPin<const N: usize, OP: OutputPin> {
    bit: u8,
    inner: *mut u8,
    parent: *mut AdvancedShiftRegister<N, OP>,
}

impl<const N: usize, OP: OutputPin> ShifterPin<N, OP> {
    pub fn test_new(
        bit: u8,
        shifter_i: usize,
        adv_shift_register: &mut AdvancedShiftRegister<N, OP>,
    ) -> Self {
        ShifterPin {
            bit,
            inner: core::ptr::addr_of_mut!(adv_shift_register.shifters[shifter_i]),
            parent: core::ptr::addr_of_mut!(*adv_shift_register),
        }
    }

    fn value<'a>(&self) -> &'a mut u8 {
        unsafe { self.inner.as_mut().unwrap() }
    }
}

impl<const N: usize, OP: OutputPin> ErrorType for ShifterPin<N, OP> {
    type Error = embedded_hal::digital::ErrorKind;
}

impl<const N: usize, OP: OutputPin> OutputPin for ShifterPin<N, OP> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        *self.value() &= !(1 << (7 - self.bit));
        unsafe {
            (*self.parent).update_shifters();
        }

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        *self.value() |= 1 << (7 - self.bit);
        unsafe {
            (*self.parent).update_shifters();
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct ShifterValueRange {
    inner: *mut [u8],
    //len: usize,
}

impl ShifterValueRange {
    pub fn set_data(&self, data: &[u8]) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr.copy_from_slice(data);
        }
    }

    pub fn data<'a>(&self) -> &'a mut [u8] {
        unsafe { self.inner.as_mut().unwrap() }
    }

    pub fn set_value(&self, index: usize, value: u8) {
        unsafe {
            let ptr = &mut *self.inner;
            ptr[index] = value;
        }
    }

    pub fn value<'a>(&self, index: usize) -> &'a mut u8 {
        unsafe {
            let ptr = &mut *self.inner;
            ptr[index].borrow_mut()
        }
    }
}
