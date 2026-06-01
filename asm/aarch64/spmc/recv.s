spmc_recv:
	sub sp, sp, #96
	stp x29, x30, [sp, #64]
	stp x20, x19, [sp, #80]
	add x29, sp, #64
	ldr x8, [x0]
	add x0, x8, #128
	add x9, x8, #256
	stp x0, xzr, [sp, #8]
	ldar x9, [x9]
	mov w10, w9
	mov x3, x9
	cmp x10, x9, lsr #32
	b.ne .LBB12_4
.LBB12_1:
	ldr x1, [x1]
	add x20, sp, #8
	add x2, x20, #8
	bl chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp x0, #2
	b.ne .LBB12_10
	ldr x8, [sp, #16]
	cbz x8, .LBB12_9
	b .LBB12_11
.LBB12_4:
	ldr x11, [x8, #560]
	ldr x12, [x8, #544]
	mov x10, x1
	and x11, x11, x9
	ldr x1, [x12, x11, lsl #3]
	dmb ishld
	ldr x12, [x8, #552]
	sub x12, x12, #1
	cmp x11, x12
	b.ne .LBB12_6
	ldr w11, [x8, #560]
	and x12, x9, #0xffffffff00000000
	orr w11, w9, w11
	add w11, w11, #1
	orr x11, x11, x12
	b .LBB12_7
.LBB12_6:
	add x11, x9, #1
.LBB12_7:
	add x8, x8, #256
	casal x3, x11, [x8]
	cmp x3, x9
	b.ne .LBB12_12
	mov x0, xzr
	ldr x8, [sp, #16]
	cbnz x8, .LBB12_11
	ldp x20, x19, [sp, #80]
	ldp x29, x30, [sp, #64]
	add sp, sp, #96
	ret
.LBB12_10:
	and x0, x0, #0x1
	ldr x8, [sp, #16]
	cbz x8, .LBB12_9
.LBB12_11:
	add x8, sp, #8
	mov x19, x0
	mov x20, x1
	add x0, x8, #8
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov x0, x19
	mov x1, x20
	ldp x20, x19, [sp, #80]
	ldp x29, x30, [sp, #64]
	add sp, sp, #96
	ret
.LBB12_12:
	mov x1, x10
	b .LBB12_1
	ldr x8, [sp, #16]
	mov x19, x0
	cbnz x8, .LBB12_15
.LBB12_14:
	mov x0, x19
	bl _Unwind_Resume
	add x0, x20, #8
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB12_14
	bl core::panicking::panic_in_cleanup
