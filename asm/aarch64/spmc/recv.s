spmc_recv:
	sub sp, sp, #96
	stp x29, x30, [sp, #64]
	stp x20, x19, [sp, #80]
	add x29, sp, #64
	ldr x8, [x0]
	add x20, sp, #8
	add x0, x8, #128
	add x9, x8, #256
	stp x0, xzr, [sp, #8]
	ldar x9, [x9]
	mov w10, w9
	mov x3, x9
	cmp x10, x9, lsr #32
	b.ne .LBB13_6
.LBB13_1:
	ldr x1, [x1]
	add x2, x20, #8
	bl chenal::channel::Chan<T,Ch,SP>::poll_acquire_slot_cold
	cmp x0, #2
	b.eq .LBB13_5
	tbz w0, #0, .LBB13_15
	mov w0, #1
.LBB13_5:
	b .LBB13_13
.LBB13_6:
	ldr x10, [x8, #560]
	ldr x11, [x8, #544]
	and x10, x10, x9
	ldr x19, [x11, x10, lsl #3]
	dmb ishld
	ldr x11, [x8, #552]
	sub x11, x11, #1
	cmp x10, x11
	b.ne .LBB13_8
	ldr w10, [x8, #560]
	and x11, x9, #0xffffffff00000000
	orr w10, w9, w10
	add w10, w10, #1
	orr x10, x10, x11
	b .LBB13_9
.LBB13_8:
	add x10, x9, #1
.LBB13_9:
	add x8, x8, #256
	casal x3, x10, [x8]
	cmp x3, x9
	b.ne .LBB13_1
	ldr x8, [sp, #8]
	add x9, x8, #288
	ldar x1, [x9]
	cmp x1, #1
	b.hi .LBB13_12
.LBB13_11:
	add x0, x8, #256
	bl spmc_waker::SpmcWaker<_,_>::wake_unsync_cold
.LBB13_12:
	mov x0, xzr
.LBB13_13:
	ldr x8, [sp, #16]
	cbnz x8, .LBB13_16
	mov x1, x19
	ldp x20, x19, [sp, #80]
	ldp x29, x30, [sp, #64]
	add sp, sp, #96
	ret
	mov x19, x1
	ldr x8, [sp, #8]
	add x9, x8, #288
	ldar x1, [x9]
	cmp x1, #1
	b.hi .LBB13_12
	b .LBB13_11
	add x8, sp, #8
	mov x20, x0
	add x0, x8, #8
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov x0, x20
	mov x1, x19
	ldp x20, x19, [sp, #80]
	ldp x29, x30, [sp, #64]
	add sp, sp, #96
	ret
	ldr x8, [sp, #16]
	mov x19, x0
	cbnz x8, .LBB13_19
.LBB13_18:
	mov x0, x19
	bl _Unwind_Resume
	add x0, x20, #8
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB13_18
	bl core::panicking::panic_in_cleanup
