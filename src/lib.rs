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

    pub fn get_pin_mut(&mut self, i: usize, bit: u8) -> ShifterPin {
        unsafe {
            ShifterPin {
                bit,
                inner: core::ptr::addr_of_mut!(self.shifters[i]),
                parent: self as *mut _ as *mut (),
                update_shifters_ptr: core::mem::transmute(
                    Self::update_shifters_trampoline
                        as unsafe extern "C" fn(*mut AdvancedShiftRegister<N, OP>),
                ),
            }
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

    unsafe extern "C" fn update_shifters_trampoline(this: *mut AdvancedShiftRegister<N, OP>) {
        (&mut *this).update_shifters();
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

type UpdateShiftersFunc = unsafe extern "C" fn(*mut ());
#[derive(Clone)]
pub struct ShifterPin {
    bit: u8,
    inner: *mut u8,

    parent: *mut (),
    update_shifters_ptr: UpdateShiftersFunc,
}

impl ShifterPin {
    /*
    pub fn test_new(
        bit: u8,
        shifter_i: usize,
        adv_shift_register: &mut AdvancedShiftRegister<N, OP>,
    ) -> Self {
        unsafe {
            ShifterPin {
                bit,
                inner: core::ptr::addr_of_mut!(adv_shift_register.shifters[shifter_i]),
                parent: adv_shift_register as *mut _ as *mut (),
                update_shifters_ptr: core::mem::transmute(
                    Self::update_shifters_trampoline
                        as unsafe extern "C" fn(*mut AdvancedShiftRegister<N, OP>),
                ),
            }
        }
    }
    */

    fn call_update_shifters(&mut self) {
        unsafe {
            (self.update_shifters_ptr)(self.parent);
        }
    }

    fn value<'a>(&self) -> &'a mut u8 {
        unsafe { self.inner.as_mut().unwrap() }
    }
}

impl ErrorType for ShifterPin {
    type Error = embedded_hal::digital::ErrorKind;
}

impl OutputPin for ShifterPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        *self.value() &= !(1 << (7 - self.bit));
        self.call_update_shifters();

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        *self.value() |= 1 << (7 - self.bit);
        self.call_update_shifters();

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
