spmc_recv:
	sub sp, sp, #96
	stp x29, x30, [sp, #64]
	stp x20, x19, [sp, #80]
	add x29, sp, #64
	ldr x8, [x0]
	add x20, sp, #8
	add x0, x8, #128
	stp x0, xzr, [sp, #8]
	ldr x3, [x8, #256]
	mov w9, w3
	cmp x9, x3, lsr #32
	b.ne .LBB12_4
.LBB12_1:
	ldr x1, [x1]
	add x2, x20, #8
	bl chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	mov w8, #2
	cmp x0, #2
	and x9, x0, #0x1
	csel x19, x8, x1, eq
	csel x0, x8, x9, eq
	ldr x8, [sp, #16]
	cbnz x8, .LBB12_10
	mov x1, x19
	ldp x20, x19, [sp, #80]
	ldp x29, x30, [sp, #64]
	add sp, sp, #96
	ret
.LBB12_4:
	ldr x9, [x8, #560]
	ldr x10, [x8, #544]
	dmb ishld
	and x9, x9, x3
	ldr x19, [x10, x9, lsl #3]
	dmb ishld
	ldr x10, [x8, #552]
	sub x10, x10, #1
	cmp x9, x10
	b.ne .LBB12_6
	ldr w9, [x8, #560]
	and x10, x3, #0xffffffff00000000
	orr w9, w3, w9
	add w9, w9, #1
	orr x9, x9, x10
	add x10, x8, #256
	mov x11, x3
	casal x11, x9, [x10]
	cmp x11, x3
	mov x3, x11
	b.eq .LBB12_7
	b .LBB12_1
.LBB12_6:
	add x9, x3, #1
	add x10, x8, #256
	mov x11, x3
	casal x11, x9, [x10]
	cmp x11, x3
	mov x3, x11
	b.ne .LBB12_1
.LBB12_7:
	add x9, x8, #416
	ldar x1, [x9]
	cmp x1, #1
	b.hi .LBB12_9
	add x0, x8, #384
	bl spmc_waker::SpmcWaker<_,_>::wake_unsync_cold
.LBB12_9:
	mov x0, xzr
	ldr x8, [sp, #16]
	cbz x8, .LBB12_3
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
	cbnz x8, .LBB12_13
.LBB12_12:
	mov x0, x19
	bl _Unwind_Resume
	add x0, x20, #8
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB12_12
	bl core::panicking::panic_in_cleanup
