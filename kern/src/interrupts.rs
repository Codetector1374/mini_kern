use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode, HandlerFunc};
use crate::arch::x86_64::descriptor_table::DOUBLE_FAULT_IST_INDEX;
use lazy_static::lazy_static;
use crate::sys::pic::ChainedPics;
use spin::MutexGuard;
use x86_64::instructions::hlt;
use crate::{FRAME_ALLOC, PAGE_TABLE};
use x86_64::structures::paging::{PageTable, Mapper, FrameAllocator, Page, PageTableFlags};
use core::borrow::BorrowMut;
use crate::memory::frame_allocator::FrameAllocWrapper;
use crate::interrupts::context_switch::{apic_timer, syscall_handler};
use crate::sys::pit::GLOBAL_PIT;
use keyboard::*;
use crate::interrupts::InterruptIndex::XHCI;
use x86_64::PrivilegeLevel;

pub mod context_switch;
mod keyboard;
mod syscall;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.general_protection_fault.set_handler_fn(gp_fault_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.divide_error.set_handler_fn(div_by_zero_handler);
        idt.overflow.set_handler_fn(overflow_handler);

        idt[InterruptIndex::Spurious.as_usize()].set_handler_fn(spurious_irq);

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::ApicTimer.as_usize()].set_handler_addr(apic_timer as u64);
        idt[InterruptIndex::XHCI.as_usize()].set_handler_addr(xhci_handler as u64);
        // Syscall
        idt[InterruptIndex::SysCall.as_usize()].set_handler_addr(syscall_handler as u64)
            .set_privilege_level(PrivilegeLevel::Ring3);

        // LAPIC Spurious
        idt[0xFF].set_handler_fn(spurious_irq);
        idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET + 0,
    Keyboard = PIC1_OFFSET + 1,
    Spurious = PIC1_OFFSET + 7,
    XHCI = PIC1_OFFSET + 11,
    ApicTimer = 0x30,
    SysCall = 0x80,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }

    pub fn as_offset(self) -> u8 {
        self.as_u8() - PIC1_OFFSET
    }
}

pub const PIC1_OFFSET: u8 = 0x20;
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn xhci_handler(_stack_frame: &mut InterruptStackFrame) {
    use crate::device::usb::interrupt::usb_interrupt_handler;
    usb_interrupt_handler();
    unsafe {PICS.lock().notify_end_of_interrupt(InterruptIndex::XHCI as u8) };
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut InterruptStackFrame, _ec: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;

    let faulting_addr = Cr2::read();
    error!("Faulting ADDR: {:?}", faulting_addr);
    error!("Error: {:?} \n{:#?}", _ec, stack_frame);
    panic!("PAGE FAULT");
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    // trace!("PIT Interrupt");
    GLOBAL_PIT.read().interrupt();
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn nmi_handler(tf: &mut InterruptStackFrame) {
    println!("NMI: {:#?}", tf);
}

extern "x86-interrupt" fn div_by_zero_handler(tf: &mut InterruptStackFrame) {
    panic!("DIV0: {:#?}", tf);
}

extern "x86-interrupt" fn overflow_handler(tf: &mut InterruptStackFrame) {
    println!("overflow: {:#?}", tf);
}

extern "x86-interrupt" fn alignment_check_handler(tf: &mut InterruptStackFrame, ec: u64) {
    println!("ALIGNMENT: EC: {}\n{:#?}",ec, tf);
}

extern "x86-interrupt" fn breakpoint_handler(tf: &mut InterruptStackFrame) {
    println!("TRAP: break\n{:#?}", tf);
}

extern "x86-interrupt" fn other_handler(tf: &mut InterruptStackFrame) {
    println!("Other: break\n{:#?}", tf);
}

extern "x86-interrupt" fn spurious_irq(_tf: &mut InterruptStackFrame) {
    trace!("Spurious IRQ7 detected");
}

extern "x86-interrupt" fn gp_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64)
{
    panic!("GPFault {}\n{:#?}", _error_code, stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}